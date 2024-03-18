use crate::time::is_date_older_than_n_seconds;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{fs, future::Future, path::PathBuf, time::SystemTime};
use tracing::info;

use crate::config_yaml::Secrets;

mod version_serde {
    use semver::Version;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Version>().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<'a> {
    last_revalidation: u64,
    pub variables: Secrets<'a>,
    #[serde(with = "version_serde")]
    pub version: Version,
}

pub struct Cache<'a> {
    pub directory: PathBuf,
    version: &'a Version,
}

impl<'a> Cache<'a> {
    pub fn new(directory: PathBuf, version: &'a Version) -> Self {
        Cache::<'a> {
            directory: directory.join("bwenv"),
            version,
        }
    }

    pub fn get(&self, profile: &str) -> Option<CacheEntry> {
        let cache_file_path = self.get_cache_file_path(profile);
        let cache_entry = std::fs::read_to_string(cache_file_path).ok()?;
        let cache_entry: CacheEntry = serde_yaml::from_str(&cache_entry).ok()?;
        Some(cache_entry)
    }

    pub async fn get_or_revalidate<'b, RevalidateFn, ReturnValue>(
        &self,
        profile: &str,
        max_age: &u64,
        revalidate: RevalidateFn,
    ) -> Option<CacheEntry>
    where
        RevalidateFn: FnOnce() -> ReturnValue,
        ReturnValue: Future<Output = Secrets<'b>>,
    {
        match self.is_stale(profile, max_age) {
            true => {
                info!(message = format!("Revalidating cache for profile {:?}", profile));
                let secrets = revalidate().await;
                self.set(profile, secrets);
                self.get(profile)
            }
            false => {
                info!(message = format!("Using cached values for profile {:?}", profile));
                self.get(profile)
            }
        }
    }

    pub fn set(&self, profile: &str, variables: Secrets) {
        let cache_file_path = self.get_cache_file_path(profile);
        fs::create_dir_all(self.directory.clone()).unwrap();
        let cache_entry = CacheEntry {
            last_revalidation: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_millis() as u64,
            version: self.version.clone(),
            variables,
        };
        let cache_entry = serde_yaml::to_string(&cache_entry).unwrap();
        std::fs::write(cache_file_path, cache_entry).unwrap();
    }

    pub fn clear(&self, profile: &str) {
        info!(message = format!("Clearing cache for profile {:?}", profile));
        let cache_file_path = self.get_cache_file_path(profile);
        let _ = fs::remove_file(cache_file_path);
    }

    pub fn invalidate(&self, profile: &str) {
        info!(message = format!("Invalidating cache for profile {:?}", profile));
        if let Some(cache_entry) = self.get(profile) {
            let cache_file_path = self.get_cache_file_path(profile);
            fs::create_dir_all(self.directory.clone()).unwrap();
            let cache_entry = CacheEntry {
                last_revalidation: 0,
                version: self.version.clone(),
                variables: cache_entry.variables,
            };
            let cache_entry = serde_yaml::to_string(&cache_entry).unwrap();
            std::fs::write(cache_file_path, cache_entry).unwrap();
        }
    }

    fn is_stale(&self, profile: &str, seconds: &u64) -> bool {
        let cache_entry = self.get(profile);
        match &cache_entry {
            None => true,
            Some(cache_entry) => {
                is_date_older_than_n_seconds(cache_entry.last_revalidation, seconds)
                    || self.version != &cache_entry.version
            }
        }
    }

    fn get_cache_file_path(&self, profile: &str) -> PathBuf {
        let mut cache_file_path = self.directory.join(profile);
        cache_file_path.set_extension("yaml");
        cache_file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{borrow::Cow, collections::HashMap};
    use tempfile::tempdir;

    fn setup_test_environment() -> (PathBuf, Version) {
        let temp_dir = tempdir().unwrap().into_path();
        let version = Version::parse("1.0.0").unwrap();
        (temp_dir, version)
    }

    #[test]
    fn test_new() {
        let (temp_dir, version) = setup_test_environment();
        let cache = Cache::new(temp_dir.clone(), &version);

        assert_eq!(cache.directory, temp_dir.join("bwenv"));
        assert_eq!(cache.version, &version);
    }

    #[tokio::test]
    async fn test_get_and_set() {
        let (temp_dir, version) = setup_test_environment();
        let cache = Cache::new(temp_dir, &version);
        let profile = "test_profile";

        let variables: HashMap<Cow<str>, Cow<str>> =
            [("key".into(), "value".into())].iter().cloned().collect();
        let secrets = Secrets(variables);

        cache.set(profile, secrets.clone());

        let cache_entry = cache.get(profile).expect("Failed to get cache entry");
        assert_eq!(cache_entry.variables, secrets);
    }

    #[tokio::test]
    async fn test_clear_and_invalidate() {
        let (temp_dir, version) = setup_test_environment();
        let cache = Cache::new(temp_dir.clone(), &version);
        let profile = "test_profile";

        let variables: HashMap<Cow<str>, Cow<str>> =
            [("key".into(), "value".into())].iter().cloned().collect();
        let secrets = Secrets(variables);

        cache.set(profile, secrets.clone());
        assert!(cache.get(profile).is_some());

        cache.clear(profile);
        assert!(cache.get(profile).is_none());

        cache.set(profile, secrets);
        cache.invalidate(profile);
        let cache_entry = cache
            .get(profile)
            .expect("Failed to get cache entry after invalidation");
        assert_eq!(cache_entry.last_revalidation, 0);
    }
}

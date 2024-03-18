use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, future::Future, path::PathBuf, time::SystemTime};
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

    pub async fn get_or_revalidate<RevalidateFn, ReturnValue>(
        &self,
        profile: &str,
        max_age: &u64,
        revalidate: RevalidateFn,
    ) -> Option<CacheEntry>
    where
        RevalidateFn: FnOnce() -> ReturnValue,
        ReturnValue: Future<Output = Vec<(String, String)>>,
    {
        match self.is_stale(profile, max_age) {
            true => {
                info!(message = format!("Revalidating cache for profile {:?}", profile));
                let secrets = revalidate().await;
                self.set(profile, &secrets);
                self.get(profile)
            }
            false => {
                info!(message = format!("Using cached values for profile {:?}", profile));
                self.get(profile)
            }
        }
    }

    pub fn set(&self, profile: &str, vars: &[(String, String)]) {
        let cache_file_path = self.get_cache_file_path(profile);
        fs::create_dir_all(self.directory.clone()).unwrap();
        let cache_entry = CacheEntry {
            last_revalidation: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_millis() as u64,
            version: self.version.clone(),
            variables: Secrets(
                vars.iter()
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .collect(),
            ),
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

fn is_date_older_than_n_seconds(unix_millis: u64, n_seconds: &u64) -> bool {
    let date_seconds = unix_millis / 1000;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");

    let threshold_time = current_time.as_secs() - n_seconds;

    date_seconds < threshold_time
}

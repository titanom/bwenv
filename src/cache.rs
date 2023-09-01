use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, future::Future, path::PathBuf, time::SystemTime};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    last_revalidation: u64,
    pub variables: BTreeMap<String, String>,
}

pub struct Cache {
    pub directory: PathBuf,
}

impl Cache {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: directory.join("bwenv"),
        }
    }

    pub fn get(&self, profile: &str) -> Option<CacheEntry> {
        let cache_file_path = self.get_cache_file_path(profile);
        let cache_entry = std::fs::read_to_string(cache_file_path).ok()?;
        let cache_entry: CacheEntry = toml::from_str(&cache_entry).ok()?;
        Some(cache_entry)
    }

    pub async fn get_or_revalidate<RevalidateFn, ReturnValue>(
        &self,
        profile: &str,
        revalidate: RevalidateFn,
    ) -> Option<CacheEntry>
    where
        RevalidateFn: FnOnce() -> ReturnValue,
        ReturnValue: Future<Output = Vec<(String, String)>>,
    {
        match self.is_stale(profile, 100) {
            true => {
                let secrets = revalidate().await;
                self.set(profile, &secrets);
                self.get(profile)
            }
            false => self.get(profile),
        }
    }

    pub fn set(&self, profile: &str, vars: &Vec<(String, String)>) -> () {
        let cache_file_path = self.get_cache_file_path(profile);
        fs::create_dir_all(self.directory.clone()).unwrap();
        let cache_entry = CacheEntry {
            last_revalidation: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_millis() as u64,
            variables: vars.to_owned().into_iter().collect(),
        };
        let cache_entry = toml::to_string(&cache_entry).unwrap();
        std::fs::write(cache_file_path, cache_entry).unwrap();
    }

    fn is_stale(&self, profile: &str, seconds: u64) -> bool {
        let cache_entry = self.get(profile);

        match cache_entry {
            None => return true,
            Some(cache_entry) => {
                is_date_older_than_n_seconds(cache_entry.last_revalidation, seconds)
            }
        }
    }

    fn get_cache_file_path(&self, profile: &str) -> PathBuf {
        let mut cache_file_path = self.directory.join(profile);
        cache_file_path.set_extension("toml");
        cache_file_path
    }

    // pub fn revalidate(&self, profile: &str) -> () {
    //     let cache_entry = self.get(profile).unwrap();
    // }
}

fn is_date_older_than_n_seconds(unix_millis: u64, n_seconds: u64) -> bool {
    let date_seconds = unix_millis / 1000;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");

    let threshold_time = current_time.as_secs() - n_seconds;

    date_seconds < threshold_time
}

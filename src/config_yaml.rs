use format_serde_error::{ErrorTypes, SerdeError};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::{error::ConfigError, fs::find_up};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMaxAge(u64);

impl Default for CacheMaxAge {
    fn default() -> Self {
        CacheMaxAge(86400)
    }
}

impl CacheMaxAge {
    pub fn as_u64(&self) -> &u64 {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachePath(PathBuf);

impl Default for CachePath {
    fn default() -> Self {
        CachePath(PathBuf::from(".cache"))
    }
}

impl CachePath {
    pub fn as_pathbuf(&self) -> &PathBuf {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    // TODO: make private
    #[serde(default)]
    pub path: CachePath,
    #[serde(default, rename = "max-age")]
    max_age: CacheMaxAge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Secrets<'a>(pub HashMap<Cow<'a, str>, Cow<'a, str>>);

impl<'a> Default for Secrets<'a> {
    fn default() -> Self {
        Secrets(HashMap::new())
    }
}

impl<'a> Secrets<'a> {
    pub fn new() -> Self {
        Secrets(HashMap::new())
    }

    pub fn merge(a: &'a Secrets<'a>, b: &'a Secrets<'a>) -> Secrets<'a> {
        let mut merged = HashMap::new();

        for (key, value) in &a.0 {
            merged.insert(Cow::Borrowed(key.as_ref()), Cow::Borrowed(value.as_ref()));
        }

        for (key, value) in &b.0 {
            merged.insert(Cow::Borrowed(key.as_ref()), Cow::Borrowed(value.as_ref()));
        }

        Secrets(merged)
    }

    pub fn as_hash_map(&mut self) -> &HashMap<Cow<'a, str>, Cow<'a, str>> {
        &self.0
    }

    pub fn as_vec(&mut self) -> Vec<(String, String)> {
        self.as_hash_map()
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Global<'a> {
    #[serde(default, rename = "overrides")]
    overrides: Secrets<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile<'a> {
    #[serde(rename = "project-id")]
    project_id: String,
    #[serde(default, rename = "overrides")]
    overrides: Secrets<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profiles<'a>(HashMap<String, Profile<'a>>);

impl<'a> Default for Profiles<'a> {
    fn default() -> Self {
        Profiles(HashMap::new())
    }
}

impl<'a> Profiles<'a> {
    pub fn get(&self, key: &str) -> Result<&Profile, ConfigError> {
        self.0.get(key).ok_or(ConfigError::NoProfile)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config<'a> {
    version: String,
    // TODO: make private
    pub cache: Cache,
    global: Option<Global<'a>>,
    profiles: Profiles<'a>,
    #[serde(skip)]
    pub path: String,
}

#[derive(Debug)]
pub struct ConfigEvaluation<'a> {
    pub version_req: VersionReq,
    pub profile_name: &'a str,
    pub project_id: &'a str,
    pub max_age: &'a CacheMaxAge,
    pub overrides: Secrets<'a>,
}

impl<'a> Config<'a> {
    pub fn new<P: AsRef<Path>>(config_file_path: P) -> Result<Self, anyhow::Error> {
        parse_config_file(config_file_path)
    }

    pub fn evaluate<'b>(
        &'b self,
        profile_name: &'b str,
    ) -> Result<ConfigEvaluation<'b>, ConfigError> {
        let profile = self.profiles.get(profile_name)?;

        let version = VersionReq::parse(&self.version).unwrap();

        let global_overrides = &self.global.as_ref().unwrap().overrides;
        let profile_overrides = &profile.overrides;

        let overrides = Secrets::merge(global_overrides, profile_overrides);

        Ok(ConfigEvaluation {
            profile_name,
            overrides,
            project_id: &profile.project_id,
            version_req: version,
            max_age: &self.cache.max_age,
        })
    }
}

fn convert_hashmap<'a>(
    input: HashMap<&'a String, &'a String>,
) -> HashMap<Cow<'a, str>, Cow<'a, str>> {
    input
        .into_iter()
        .map(|(k, v)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(v.as_str())))
        .collect()
}

fn merge_hashmaps<'a, K, V>(
    map1: &'a HashMap<K, V>,
    map2: &'a HashMap<K, V>,
) -> HashMap<&'a K, &'a V>
where
    K: Eq + std::hash::Hash + 'a,
    V: 'a,
{
    let mut merged = HashMap::new();

    for (key, value) in map1 {
        merged.insert(key, value);
    }

    for (key, value) in map2 {
        merged.insert(key, value);
    }

    merged
}

pub fn find_local_config() -> Result<PathBuf, ConfigError> {
    ["bwenv.yaml", "bwenv.yml"]
        .iter()
        .filter_map(|filename| find_up(filename, None))
        .next()
        .ok_or(ConfigError::NotFound)
}

fn parse_config_file<'a, P: AsRef<Path>>(file_path: P) -> Result<Config<'a>, anyhow::Error> {
    let mut raw = String::new();
    let mut file = File::open(file_path)
        .map_err(|_| ConfigError::Read)
        .unwrap();
    file.read_to_string(&mut raw);

    Ok(serde_yaml::from_str::<Config>(&raw)
        .map_err(|err| SerdeError::new(raw.to_string(), ErrorTypes::Yaml(err)))?)
}

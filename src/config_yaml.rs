use format_serde_error::{ErrorTypes, SerdeError};
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{
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
struct OverrideValues(HashMap<String, String>);

impl Default for OverrideValues {
    fn default() -> Self {
        OverrideValues(HashMap::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Global {
    #[serde(default, rename = "override-values")]
    override_values: OverrideValues,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    #[serde(rename = "project-id")]
    project_id: String,
    #[serde(default, rename = "override-values")]
    override_values: OverrideValues,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profiles(HashMap<String, Profile>);

impl Default for Profiles {
    fn default() -> Self {
        Profiles(HashMap::new())
    }
}

impl Profiles {
    pub fn get(&self, key: &str) -> Result<&Profile, ConfigError> {
        self.0.get(key).ok_or(ConfigError::NoProfile)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    version: String,
    // TODO: make private
    pub cache: Cache,
    global: Option<Global>,
    profiles: Profiles,
    #[serde(skip)]
    pub path: String,
}

#[derive(Debug)]
pub struct ConfigEvaluation<'a> {
    pub version_req: VersionReq,
    pub profile_name: &'a str,
    pub project_id: &'a str,
    pub max_age: &'a CacheMaxAge,
}

impl Config {
    pub fn new<P: AsRef<Path>>(config_file_path: P) -> Result<Self, anyhow::Error> {
        parse_config_file(config_file_path)
    }

    pub fn evaluate<'a>(
        &'a self,
        profile_name: &'a str,
    ) -> Result<ConfigEvaluation<'a>, ConfigError> {
        let profile = self.profiles.get(profile_name)?;

        let version = VersionReq::parse(&self.version).unwrap();

        Ok(ConfigEvaluation {
            profile_name,
            project_id: &profile.project_id,
            version_req: version,
            max_age: &self.cache.max_age,
        })
    }
}

pub fn find_local_config() -> Result<PathBuf, ConfigError> {
    ["bwenv.yaml", "bwenv.yml"]
        .iter()
        .filter_map(|filename| find_up(filename, None))
        .next()
        .ok_or(ConfigError::NotFound)
}

fn parse_config_file<P: AsRef<Path>>(file_path: P) -> Result<Config, anyhow::Error> {
    let mut raw = String::new();
    let mut file = File::open(file_path)
        .map_err(|_| ConfigError::Read)
        .unwrap();
    file.read_to_string(&mut raw);

    Ok(serde_yaml::from_str::<Config>(&raw)
        .map_err(|err| SerdeError::new(raw.to_string(), ErrorTypes::Yaml(err)))?)
}

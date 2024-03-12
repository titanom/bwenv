use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{error::ConfigError, fs::find_up};

#[derive(Debug, Serialize, Deserialize)]
struct CacheMaxAge(u32);

impl Default for CacheMaxAge {
    fn default() -> Self {
        CacheMaxAge(86400)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachePath(String);

impl Default for CachePath {
    fn default() -> Self {
        CachePath(String::from(".cache"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    #[serde(default)]
    path: String,
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
    name: String,
    #[serde(rename = "project-id")]
    project_id: String,
    #[serde(rename = "override-values")]
    override_values: OverrideValues,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profiles(Vec<Profile>);

impl Default for Profiles {
    fn default() -> Self {
        Profiles(Vec::new())
    }
}

impl Profiles {
    pub fn iter(&self) -> std::slice::Iter<'_, Profile> {
        self.0.iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    version: String,
    cache: Cache,
    global: Option<Global>,
    profiles: Profiles,
}

#[derive(Debug)]
pub struct ConfigEvaluation<'a> {
    pub version_req: &'a VersionReq,
    pub profile_name: &'a str,
    pub project_id: &'a str,
    pub max_age: &'a CacheMaxAge,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config_file_path = find_local_config()?;
        parse_config_file(config_file_path)
    }

    pub fn evaluate(&self, profile: Option<&str>) -> Result<ConfigEvaluation, ConfigError> {
        let max_age = self.cache.max_age;

        let profile_name = profile.unwrap_or_else(|| get_profile_from_env(&Vec::new()).unwrap());

        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.name == profile_name)
            .ok_or(ConfigError::NoProfile)?;

        let project_id = profile.project_id;

        let version = VersionReq::parse(&self.version).unwrap();

        Ok(ConfigEvaluation {
            profile_name: &profile.name,
            project_id: &project_id,
            version_req: &version,
            max_age: &max_age,
        })
    }
}

fn find_local_config() -> Result<PathBuf, ConfigError> {
    ["bwenv.yaml", "bwenv.yml"]
        .iter()
        .filter_map(|filename| find_up(filename, None))
        .next()
        .ok_or(ConfigError::NotFound)
}

fn parse_config_file<P: AsRef<Path>>(file_path: P) -> Result<Config, ConfigError> {
    let file = File::open(file_path).map_err(|_| ConfigError::Read)?;
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).map_err(|_| ConfigError::Invalid)
}

fn get_profile_from_env(env_var_names: &Vec<String>) -> Option<&str> {
    let mut existing_env_vars = Vec::new();

    for env_var_name in env_var_names {
        if let Ok(env_var_value) = env::var(env_var_name) {
            existing_env_vars.push(env_var_value);
        }
    }

    existing_env_vars.first().map(|x| x.as_str())
}

use anyhow::anyhow;
use format_serde_error::{ErrorTypes, SerdeError};
use semver::VersionReq;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::config_yaml::{self, Profiles};

use crate::error::{ConfigError, Error};

type Override = Option<BTreeMap<String, String>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Profile<'a> {
    pub project: Option<String>,
    pub environment: Option<String>,
    #[serde(default)]
    pub r#override: config_yaml::Secrets<'a>,
}

#[derive(Debug, Deserialize)]
pub struct Cache {
    #[serde(default)]
    pub max_age: config_yaml::CacheMaxAge,
    pub path: config_yaml::CachePath,
}

#[derive(Debug, Deserialize)]
pub struct Config<'a> {
    pub version: String,
    pub environment: Option<Vec<String>>,
    pub cache: Cache,
    pub project: Option<String>,
    pub r#override: config_yaml::Secrets<'a>,
    pub profile: BTreeMap<String, Profile<'a>>,
    #[serde(skip)]
    pub path: String,
}

pub struct ConfigEvaluation<'a> {
    pub version_req: VersionReq,
    pub profile_name: String,
    pub project_id: String,
    pub max_age: config_yaml::CacheMaxAge,
    pub r#override: config_yaml::Secrets<'a>,
}

fn convert_toml_profile_to_yaml_profile<'a>(toml_profile: Profile<'a>) -> config_yaml::Profile<'a> {
    config_yaml::Profile {
        project_id: toml_profile.project.unwrap(),
        overrides: toml_profile.r#override,
    }
}

impl Config<'_> {
    pub fn new<P: AsRef<Path>>(config_file_path: P) -> anyhow::Result<Self> {
        if config_file_path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            != Some("toml")
        {
            return Err(anyhow!("Configuration file must be a .toml file"));
        }

        let mut config = parse_config_file(config_file_path)?;
        config.profile.insert(
            String::from("default"),
            Profile {
                environment: None,
                project: config.project.clone(),
                r#override: config.r#override.clone(),
            },
        );
        Ok(config)
    }

    pub fn as_yaml_config<'a>(&'a self) -> crate::config_yaml::Config<'_> {
        crate::config_yaml::Config::<'a> {
            version: self.version.clone(),
            path: self.path.clone(),
            global: Some(config_yaml::Global {
                overrides: self.r#override.clone(),
            }),
            profiles: Profiles::new(
                <BTreeMap<std::string::String, Profile<'_>> as Clone>::clone(&self.profile)
                    .into_iter()
                    .map(|(key, toml_profile)| {
                        let yaml_profile = convert_toml_profile_to_yaml_profile(toml_profile);
                        (key, yaml_profile)
                    })
                    .collect(),
            ),
            cache: config_yaml::Cache {
                path: config_yaml::CachePath(self.cache.path.as_pathbuf().clone()),
                max_age: config_yaml::CacheMaxAge(self.cache.max_age.as_u64().clone()),
            },
        }
    }
}

fn find_up(filename: &str, max_parents: Option<i32>) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut current_path = current_dir.as_path();

    for _ in 0..max_parents.unwrap_or(10) {
        let file_path = current_path.join(filename);

        if file_path.exists() {
            return Some(file_path);
        }

        match current_path.parent() {
            Some(parent) => current_path = parent,
            None => break,
        }
    }

    None
}

fn parse_config_file<'a, P: AsRef<Path>>(file_path: P) -> Result<Config<'a>, anyhow::Error> {
    let mut raw = String::new();
    let mut file = File::open(file_path)
        .map_err(|_| ConfigError::Read)
        .unwrap();
    file.read_to_string(&mut raw);

    Ok(toml::from_str::<Config>(&raw)
        .map_err(|err| SerdeError::new(raw.to_string(), ErrorTypes::Toml(err)))?)
}

fn find_local_config() -> Option<PathBuf> {
    find_up("bwenv.toml", None)
}

fn get_profile_from_env(env_var_names: &Vec<String>) -> Option<String> {
    let mut existing_env_vars = Vec::new();

    for env_var_name in env_var_names {
        if let Ok(env_var_value) = env::var(env_var_name) {
            existing_env_vars.push(env_var_value);
        }
    }

    existing_env_vars.first().map(|s| s.to_string())
}

use anyhow::anyhow;
use format_serde_error::{ErrorTypes, SerdeError};
use serde::Deserialize;
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};
use tracing::info;

use crate::config_yaml::{self, Profiles};

use crate::error::ConfigError;
use crate::schema_types::VersionReq;

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
    pub version: VersionReq,
    pub environment: Option<Vec<String>>,
    pub cache: Cache,
    pub project: Option<String>,
    pub r#override: config_yaml::Secrets<'a>,
    pub profile: BTreeMap<String, Profile<'a>>,
    #[serde(skip)]
    pub path: String,
}

fn convert_toml_profile_to_yaml_profile(toml_profile: Profile<'_>) -> config_yaml::Profile<'_> {
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
                overrides: config_yaml::GlobalOverrides(self.r#override.clone()),
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
                path: config_yaml::CachePath(self.cache.path.clone()),
                max_age: config_yaml::CacheMaxAge(*self.cache.max_age),
            },
        }
    }
}

fn parse_config_file<'a, P: AsRef<Path>>(file_path: P) -> Result<Config<'a>, anyhow::Error> {
    if let Some(path) = file_path.as_ref().to_str() {
        info!(message = format!("Using configuration file at {:?}", path));
    }
    let mut raw = String::new();
    let mut file = File::open(file_path)
        .map_err(|_| ConfigError::Read)
        .unwrap();
    let _ = file.read_to_string(&mut raw);

    Ok(toml::from_str::<Config>(&raw)
        .map_err(|err| SerdeError::new(raw.to_string(), ErrorTypes::Toml(err)))?)
}

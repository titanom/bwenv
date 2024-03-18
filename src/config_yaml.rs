use format_serde_error::{ErrorTypes, SerdeError};
use semver::VersionReq;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use tracing::info;

use crate::error::ConfigError;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMaxAge(pub u64);

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
pub struct CachePath(pub PathBuf);

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
    pub max_age: CacheMaxAge,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secrets<'a>(pub HashMap<Cow<'a, str>, Cow<'a, str>>);

impl<'a> Default for Secrets<'a> {
    fn default() -> Self {
        Secrets(HashMap::new())
    }
}

impl<'a> Secrets<'a> {
    pub fn as_hash_map(&self) -> &HashMap<Cow<'a, str>, Cow<'a, str>> {
        &self.0
    }

    pub fn merge(a: &'a Secrets<'a>, b: &'a Secrets<'a>) -> Secrets<'a> {
        Secrets(
            a.as_hash_map()
                .iter()
                .chain(b.as_hash_map().iter())
                .map(|(k, v)| (Cow::Borrowed(k.as_ref()), Cow::Borrowed(v.as_ref())))
                .collect(),
        )
    }

    pub fn as_vec(&mut self) -> Vec<(String, String)> {
        self.as_hash_map()
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    pub fn table(&self, reveal: bool) -> String {
        let mut table = Table::new("{:>} :: {:<}");
        let map = self.as_hash_map();
        for (key, value) in map.into_iter() {
            table.add_row(Row::new().with_cell(key).with_cell(if reveal {
                value
            } else {
                "**redacted**"
            }));
        }
        table.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Global<'a> {
    #[serde(
        default,
        rename = "overrides",
        deserialize_with = "deserialize_null_default"
    )]
    pub overrides: Secrets<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile<'a> {
    #[serde(rename = "project-id")]
    pub project_id: String,
    #[serde(
        default,
        rename = "overrides",
        deserialize_with = "deserialize_null_default"
    )]
    pub overrides: Secrets<'a>,
}

type ProfilesMap<'a> = HashMap<String, Profile<'a>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profiles<'a>(ProfilesMap<'a>);

impl<'a> Default for Profiles<'a> {
    fn default() -> Self {
        Profiles(HashMap::new())
    }
}

impl<'a> Profiles<'a> {
    pub fn new(hash_map: ProfilesMap<'a>) -> Self {
        Self(hash_map)
    }

    pub fn get(&self, key: &str) -> Result<&Profile, ConfigError> {
        self.0.get(key).ok_or(ConfigError::NoProfile)
    }
}

#[derive(Debug, Serialize, Deserialize)]
// TODO: make fields private
pub struct Config<'a> {
    pub version: String,
    pub cache: Cache,
    pub global: Option<Global<'a>>,
    pub profiles: Profiles<'a>,
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

        info!(message = format!("Using profile {:?}", profile_name));

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

fn parse_config_file<'a, P: AsRef<Path>>(file_path: P) -> Result<Config<'a>, anyhow::Error> {
    info!(message = format!("Using configuration file at {:?}", file_path.as_ref()));
    let mut raw = String::new();
    let mut file = File::open(file_path)
        .map_err(|_| ConfigError::Read)
        .unwrap();
    let _ = file.read_to_string(&mut raw);

    Ok(serde_yaml::from_str::<Config>(&raw)
        .map_err(|err| SerdeError::new(raw.to_string(), ErrorTypes::Yaml(err)))?)
}

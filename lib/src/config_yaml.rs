use colored::Colorize;
use format_serde_error::{ErrorTypes, SerdeError};
use schemars::JsonSchema;
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

use crate::{error::ConfigError, schema_types::VersionReq};

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct Cache {
    #[serde(default)]
    #[schemars(
        title = "Cache Path",
        description = "Path to the local secrets cache directory relative to the project root"
    )]
    pub path: CachePath,

    #[serde(default, rename = "max-age")]
    #[schemars(
        title = "Cache Max Age",
        description = "Maximum age of the local secrets cache in seconds"
    )]
    pub max_age: CacheMaxAge,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, JsonSchema)]
pub struct Secrets<'a>(pub HashMap<Cow<'a, str>, Cow<'a, str>>);

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, JsonSchema)]
pub struct GlobalOverrides<'a>(pub Secrets<'a>);

impl<'a> FromIterator<(String, String)> for Secrets<'a> {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        let mut map = HashMap::new();
        for (key, value) in iter {
            map.insert(Cow::Owned(key), Cow::Owned(value));
        }
        Secrets(map)
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
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    pub fn table(&self, reveal: bool) -> String {
        let mut table = Table::new("{:>} :: {:<}");
        let map = self.as_hash_map();
        for (key, value) in map.iter() {
            table.add_row(Row::new().with_cell(key).with_cell(if reveal {
                value.normal()
            } else {
                "**redacted**".italic().dimmed()
            }));
        }
        table.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Global", description = "Global configuration options")]
pub struct Global<'a> {
    #[serde(
        default,
        rename = "overrides",
        deserialize_with = "deserialize_null_default"
    )]
    #[schemars(
        title = "Global Overrides",
        description = "Overrides that apply to all profiles unless specified by the profile itself"
    )]
    pub overrides: GlobalOverrides<'a>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "Profile", description = "Configuration for a single profile")]
pub struct Profile<'a> {
    #[schemars(
        title = "Profile Bitwarden Project ID",
        description = "ID of the Bitwarden project"
    )]
    #[serde(rename = "project-id")]
    pub project_id: String,

    #[schemars(
        title = "Profile Overrides",
        description = "Profile-specific secret overrides"
    )]
    #[serde(
        default,
        rename = "overrides",
        deserialize_with = "deserialize_null_default"
    )]
    pub overrides: Secrets<'a>,
}

type ProfilesMap<'a> = HashMap<String, Profile<'a>>;

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct Profiles<'a>(ProfilesMap<'a>);

impl<'a> Profiles<'a> {
    pub fn new(hash_map: ProfilesMap<'a>) -> Self {
        Self(hash_map)
    }

    pub fn get(&self, key: &str) -> Result<&Profile, ConfigError> {
        self.0.get(key).ok_or(ConfigError::NoProfile)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Config<'a> {
    #[schemars(
        title = "Version",
        description = "A semantic version that the version of the bwenv CLI must match"
    )]
    pub version: VersionReq,

    #[schemars(
        title = "Cache",
        description = "Options related to the local secrets cache"
    )]
    pub cache: Cache,

    #[schemars(
        title = "Global",
        description = "Overrides for global configuration options, applied to all profiles"
    )]
    pub global: Option<Global<'a>>,

    #[schemars(
        title = "Profiles",
        description = "List of profiles that hold information about the bitwarden project and profile-specific overrides"
    )]
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

        let global_overrides = &self.global.as_ref().unwrap().overrides;
        let profile_overrides = &profile.overrides;

        let overrides = Secrets::merge(&global_overrides.0, profile_overrides);

        Ok(ConfigEvaluation {
            profile_name,
            overrides,
            project_id: &profile.project_id,
            version_req: self.version.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_config_file_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
version: "1.0.0"
cache:
  path: "/tmp/cache"
  max-age: 86400
global:
  overrides: {{}}
profiles: {{}}
"#
        )
        .unwrap();

        let config = parse_config_file(temp_file.path());
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.version, VersionReq::parse("1.0.0").unwrap());
        assert_eq!(
            config.cache.path.as_pathbuf().to_str().unwrap(),
            "/tmp/cache"
        );
        assert_eq!(*config.cache.max_age.as_u64(), 86400);
    }

    #[test]
    fn test_config_evaluate_profile_not_found() {
        let config = Config {
            version: VersionReq::parse("1.0.0").unwrap(),
            cache: Cache::default(),
            global: None,
            profiles: Profiles::default(),
            path: String::new(),
        };

        let result = config.evaluate("nonexistent");
        assert!(matches!(result, Err(ConfigError::NoProfile)));
    }

    #[test]
    fn test_config_evaluate_with_overrides() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
version: "1.0.0"
cache:
  path: "/tmp/cache"
  max-age: 86400
global:
  overrides:
    global_key: "global_value"
profiles:
  test_profile:
    project-id: "test_project"
    overrides:
      profile_key: "profile_value"
      global_key: "overridden_global_value"
"#
        )
        .unwrap();

        let config = parse_config_file(temp_file.path()).unwrap();
        let eval_result = config.evaluate("test_profile").unwrap();

        assert_eq!(eval_result.profile_name, "test_profile");
        assert_eq!(eval_result.project_id, "test_project");
        assert_eq!(
            eval_result.overrides.0.get("global_key").unwrap(),
            "overridden_global_value"
        );
        assert_eq!(
            eval_result.overrides.0.get("profile_key").unwrap(),
            "profile_value"
        );
    }

    #[test]
    fn test_global_overrides_without_profile() {
        let config = Config {
            version: VersionReq::parse("1.0.0").unwrap(),
            cache: Cache::default(),
            global: Some(Global {
                overrides: Secrets(
                    [("global_key".into(), "global_value".into())]
                        .iter()
                        .cloned()
                        .collect(),
                ),
            }),
            profiles: Profiles::default(),
            path: String::new(),
        };

        let eval_result = config.evaluate("nonexistent").err().unwrap();
        assert!(matches!(eval_result, ConfigError::NoProfile));
    }
}

use serde::Deserialize;
use std::{collections::BTreeMap, env, fs::File, io::Read, path::PathBuf};

use crate::cli::Args;
use crate::error::Error;

type Overrides = Option<Vec<BTreeMap<String, String>>>;

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub project: Option<String>,
    pub environment: Option<String>,
    pub overrides: Overrides,
}

#[derive(Debug, Deserialize)]
pub struct Cache {
    pub max_age: Option<u64>,
    // stale_while_revalidate: Option<u64>,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub environment: Option<Vec<String>>,
    pub cache: Cache,
    pub project: Option<String>,
    pub profiles: BTreeMap<String, Profile>,
    #[serde(skip)]
    pub path: String,
}

pub struct ConfigEvaluation {
    pub profile_name: String,
    pub project_id: String,
    pub max_age: u64,
    pub overrides: Overrides,
}

impl Config {
    pub fn new() -> Self {
        let config_file_path = find_local_config().unwrap();
        let mut config = parse_config_file(&config_file_path).unwrap();
        config.profiles.insert(
            String::from("no_profile"),
            Profile {
                environment: None,
                project: config.project.to_owned(),
                overrides: None,
            },
        );
        config
    }

    pub fn evaluate(&self, cli_args: &Args) -> Result<ConfigEvaluation, Error> {
        let max_age = self.cache.max_age.unwrap_or(86400);

        let default_environment: Vec<String> = std::vec::Vec::new();
        let env_var_names = self.environment.as_ref().unwrap_or(&default_environment);

        let profile_name = match cli_args.profile.clone() {
            Some(profile) => profile,
            None => get_profile_from_env(env_var_names).unwrap_or(String::from("no_profile")),
        };

        let profile = self
            .profiles
            .get(&profile_name)
            .ok_or(Error::ProfileNotConfigured)?;

        let project = profile
            .project
            .as_ref()
            .or_else(|| {
                log::error!("could not find project configuration");
                std::process::exit(1);
            })
            .unwrap();

        Ok(ConfigEvaluation {
            profile_name: profile_name.to_string(),
            project_id: project.to_string(),
            max_age,
            overrides: profile.overrides.clone(),
        })
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

fn parse_config_file(file_path: &PathBuf) -> Result<Config, Error> {
    let mut toml_str = String::new();
    let mut file = File::open(file_path).unwrap();
    file.read_to_string(&mut toml_str).unwrap();

    let mut config: Config = toml::from_str(&toml_str).unwrap();
    config.path = file_path.to_str().unwrap().to_owned();

    Ok(config)
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

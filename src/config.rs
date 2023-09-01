use serde::Deserialize;
use std::{collections::BTreeMap, env, fs::File, io::Read, path::PathBuf};
use toml::de::Error;

#[derive(Debug)]
enum Preset {
    Node,
    Python,
}

impl<'de> Deserialize<'de> for Preset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "node" => Ok(Preset::Node),
            "python" => Ok(Preset::Python),
            _ => Err(serde::de::Error::custom(format!("unknown preset: {}", s))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub project: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Cache {
    // max_age: Option<u64>,
    // stale_while_revalidate: Option<u64>,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub environment: Option<Vec<String>>,
    pub cache: Cache,
    // preset: Option<Preset>,
    pub project: Option<String>,
    #[serde(flatten)]
    pub profiles: BTreeMap<String, Profile>,
}

pub struct ConfigEvaluation {
    pub profile_name: String,
    pub project_id: String,
}

#[derive(Debug)]
pub enum ConfigEvaluationError {
    NoProfile,
}

impl Config {
    pub fn new() -> Self {
        let config_file_path = find_local_config().unwrap();
        parse_config_file(&config_file_path).unwrap()
    }

    pub fn evaluate(&self) -> Result<ConfigEvaluation, ConfigEvaluationError> {
        let env_var_names = self
            .environment
            .as_ref()
            .ok_or_else(|| ConfigEvaluationError::NoProfile)?;

        let profile_name =
            get_profile_from_env(env_var_names).ok_or_else(|| ConfigEvaluationError::NoProfile)?;

        let profile = self
            .profiles
            .get(&profile_name)
            .ok_or_else(|| ConfigEvaluationError::NoProfile)?;

        let project = profile.project.as_ref().unwrap_or_else(|| {
            &self
                .project
                .as_ref()
                .expect("please provide a project via environment variables or config file")
        });

        Ok(ConfigEvaluation {
            profile_name: profile_name.to_string(),
            project_id: project.to_string(),
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

    let config: Config = toml::from_str(&toml_str).unwrap();

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

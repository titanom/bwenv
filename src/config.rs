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
struct Environment {
    project: Option<String>,
    environment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cache {
    max_age: Option<u64>,
    stale_while_revalidate: Option<u64>,
    directory: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    environment: Option<Vec<String>>,
    cache: Option<Cache>,
    preset: Option<Preset>,
    project: Option<String>,
    #[serde(flatten)]
    environments: BTreeMap<String, Environment>,
}

pub fn find_up(filename: &str, max_parents: Option<i32>) -> Option<PathBuf> {
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

pub fn parse_config_file(file_path: &PathBuf) -> Result<Config, Error> {
    let mut toml_str = String::new();
    let mut file = File::open(file_path).unwrap();
    file.read_to_string(&mut toml_str).unwrap();

    let config: Config = toml::from_str(&toml_str).unwrap();
    println!("config: {:?}", config);
    Ok(config)
}

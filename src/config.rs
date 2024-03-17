use crate::error::ConfigError;
use crate::fs::find_up;
use anyhow;
use std::path::PathBuf;

pub enum LocalConfig {
    Yaml(PathBuf),
    Toml(PathBuf),
}

impl LocalConfig {
    pub fn as_pathbuf(&self) -> &PathBuf {
        match self {
            Self::Yaml(path) => path,
            Self::Toml(path) => path,
        }
    }
}

pub fn find_local_config() -> anyhow::Result<LocalConfig, ConfigError> {
    let yaml_config = ["bwenv.yaml", "bwenv.yml"]
        .iter()
        .find_map(|filename| find_up(filename, None));

    if let Some(path) = yaml_config {
        return Ok(LocalConfig::Yaml(path));
    }

    let toml_config = find_up("bwenv.toml", None);

    if let Some(path) = toml_config {
        return Ok(LocalConfig::Toml(path));
    }

    Err(ConfigError::NotFound)
}

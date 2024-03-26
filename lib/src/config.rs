use crate::error::ConfigError;
use crate::fs::find_up;

use std::path::{Path, PathBuf};

#[derive(Debug)]
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

pub fn find_local_config(cwd: Option<&Path>) -> anyhow::Result<LocalConfig, ConfigError> {
    let yaml_config = ["bwenv.yaml", "bwenv.yml"]
        .iter()
        .find_map(|filename| find_up(filename, None, cwd));

    if let Some(path) = yaml_config {
        return Ok(LocalConfig::Yaml(path));
    }

    let toml_config = find_up("bwenv.toml", None, cwd);

    if let Some(path) = toml_config {
        return Ok(LocalConfig::Toml(path));
    }

    Err(ConfigError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper function to write a dummy config file
    fn create_config_file(dir: &Path, filename: &str) {
        let file_path = dir.join(filename);
        let mut file = File::create(file_path).expect("Failed to create test config file.");
        writeln!(file, "name: TestConfig").expect("Failed to write to test config file.");
    }

    #[test]
    fn finds_yaml_config_in_current_dir() {
        let temp_dir = tempdir().unwrap();
        create_config_file(temp_dir.path(), "bwenv.yaml");

        let result = find_local_config(Some(temp_dir.path()));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(matches!(config, LocalConfig::Yaml(_)));
    }

    #[test]
    fn finds_toml_config_in_parent_dir() {
        let temp_dir = tempdir().unwrap();
        let child_dir = temp_dir.path().join("child");
        std::fs::create_dir(&child_dir).unwrap();
        create_config_file(temp_dir.path(), "bwenv.toml");

        let result = find_local_config(Some(&child_dir));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(matches!(config, LocalConfig::Toml(_)));
    }

    #[test]
    fn config_not_found_returns_error() {
        let temp_dir = tempdir().unwrap();

        let result = find_local_config(Some(temp_dir.path()));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ConfigError::NotFound));
    }
}

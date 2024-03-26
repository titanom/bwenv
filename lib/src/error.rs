use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    Read,
    NotFound,
    NoProfile,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Read => write!(f, "Error reading configuration file"),
            ConfigError::NotFound => write!(f, "Configuration file not found"),
            ConfigError::NoProfile => write!(f, "Profile not found in the configuration"),
        }
    }
}

impl error::Error for ConfigError {}

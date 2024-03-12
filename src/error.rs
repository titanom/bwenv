#[derive(Debug, PartialEq)]
pub enum Error {
    _NoProfileInput,
    ProfileNotConfigured,
    _ProjectNotConfigured,
    _RemoteProjectNotFound,
    _CacheDirNorConfigured,
    InvalidConfigurationFile,
    ReadConfigurationFile,
    _NoConfigurationFile,
}

#[derive(Debug, PartialEq)]
pub enum BwenvError {
    Config(ConfigError),
}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    Read,
    Parse,
    Invalid,
    NotFound,
    NoProfile,
}

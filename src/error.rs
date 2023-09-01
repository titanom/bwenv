#[derive(Debug)]
pub enum Error {
    NoProfileInput,
    ProfileNotConfigured,
    _ProjectNotConfigured,
    _RemoteProjectNotFound,
    _CacheDirNorConfigured,
    _InvalidConfigurationFile,
    _NoConfigurationFile,
}

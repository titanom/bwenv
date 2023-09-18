#[derive(Debug)]
pub enum Error {
    _NoProfileInput,
    ProfileNotConfigured,
    _ProjectNotConfigured,
    _RemoteProjectNotFound,
    _CacheDirNorConfigured,
    _InvalidConfigurationFile,
    _NoConfigurationFile,
}

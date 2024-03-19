#[derive(Debug, PartialEq)]
pub enum ConfigError {
    Read,
    NotFound,
    NoProfile,
}

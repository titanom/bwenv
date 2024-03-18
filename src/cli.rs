use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(last = true)]
    pub slop: Vec<String>,

    #[arg(
        short,
        long,
        long_help = "access token for the service account",
        help = "access token for the service account",
        env = "BWS_ACCESS_TOKEN",
        required = false,
        hide_env_values = true
    )]
    pub token: String,

    #[arg(
        short,
        long,
        long_help = "profile for loading project configuration",
        help = "profile for loading project configuration",
        env = "BWENV_PROFILE",
        required = false
    )]
    pub profile: Option<String>,

    #[arg(
        short,
        long,
        value_enum,
        default_value_t = LogLevel::Info,
        help = "Set the log level",
        env = "BWENV_LOG_LEVEL",
        required = false
    )]
    pub log_level: LogLevel,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_tracing_env(&self) -> String {
        match self {
            LogLevel::Error => "error".to_string(),
            LogLevel::Warn => "warn".to_string(),
            LogLevel::Info => "info".to_string(),
            LogLevel::Debug => "debug".to_string(),
            LogLevel::Trace => "trace".to_string(),
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Cache(CacheCommand),

    Inspect(InspectArgs),
}

#[derive(Subcommand, Debug)]
pub enum CacheCommand {
    /// clear the cache of a given profile
    Clear,

    /// invalidate the cache of a given profile
    Invalidate,
}

#[derive(Parser, Debug)]
pub struct InspectArgs {
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "reveal secrets in output",
        long_help = "reveal secrets in output"
    )]
    pub reveal: bool,
}

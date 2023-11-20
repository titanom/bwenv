use clap::{Parser, Subcommand};

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
        required = false
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
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Cache(CacheCommand),
}

#[derive(Subcommand, Debug)]
pub enum CacheCommand {
    /// clear the cache of a given profile
    Clear,

    /// invalidate the cache of a given profile
    Invalidate,
}

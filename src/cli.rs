use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        long_help = "Access token for the service account",
        env = "BWS_ACCESS_TOKEN"
    )]
    pub token: String,

    #[arg(
        short,
        long,
        long_help = "Secret manager project name",
        required = true
    )]
    pub project: String,

    #[arg(short, long, long_help = "Profile of the project", required = true)]
    pub profile: String,

    #[arg(
        short,
        long,
        long_help = "Cache directory for the secrets",
        required = true
    )]
    pub cache_dir: String,

    #[arg(
        short,
        long,
        long_help = "Revalidate the cache after the giben number of seconds",
        default_value_t = 3600
    )]
    pub revalidate: u64,
}

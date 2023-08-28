use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        long_help = "Access token for the service account",
        env = "BWS_ACCESS_TOKEN"
    )]
    token: String,

    #[arg(short, long, long_help = "Secret manager project name", required = true)]
    project: String,

    #[arg(short, long, long_help = "Environment of the project", required = true)]
    environment: String,

    #[arg(short, long, long_help = "Cache directory for the secrets", required = true)]
    cache_dir: String,

    #[arg(
        short,
        long,
        long_help = "Revalidate the cache after the giben number of seconds",
        default_value_t = 3600
    )]
    revalidate: u64,
}

fn main() {
    let args = Args::parse();

    println!("{:?}", args)
}

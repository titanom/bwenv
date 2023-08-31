use std::process::{Command, Stdio};

use clap::{Args as ClapArgs, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        long_help = "Access token for the service account",
        env = "BWS_ACCESS_TOKEN",
        required = false
    )]
    pub token: String,

    #[arg(last = true)]
    pub slop: Vec<String>,
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Secret manager project name",
    //     required = false
    // )]
    // pub project: String,

    // #[arg(long, long_help = "Profile of the project", required = false)]
    // pub profile: String,
    //
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Cache directory for the secrets",
    //     required = false
    // )]
    // pub cache_dir: String,
    //
    // #[arg(
    //     short,
    //     long,
    //     long_help = "Revalidate the cache after the giben number of seconds",
    //     default_value_t = 3600
    // )]
    // pub revalidate: u64,
}

pub struct CLI {
    args: Args,
}

impl CLI {
    pub fn new() -> Self {
        CLI {
            args: Args::parse(),
        }
    }

    pub fn get_program(&self) -> (String, Vec<String>) {
        let slop = &self.args.slop;
        let program = &slop[0];
        let args = slop[1..].to_vec();

        (program.to_owned(), args)
    }
}

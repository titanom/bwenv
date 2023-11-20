use clap::Parser;
use cli::CacheCommand;
use log::Level;
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Command, Stdio},
};

mod bitwarden;
mod cache;
mod cli;
mod config;
mod error;

use cache::CacheEntry;
// use cli::Args;
use config::ConfigEvaluation;

use crate::cache::Cache;
use crate::config::Config;
use crate::{bitwarden::BitwardenClient, cli::Cli};

// use clap_markdown;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _ = simple_logger::init_with_level(Level::Error);

    // generate docs
    // let markdown: String = clap_markdown::help_markdown::<Args>();
    // println!("{}", markdown);
    let cli = Cli::parse();

    pub fn get_program(cli: &Cli) -> Option<(String, Vec<String>)> {
        let slop = &cli.slop;
        match &slop.get(0) {
            Some(program) => {
                let args = slop[1..].to_vec();

                Some((program.to_string(), args))
            }
            None => None,
        }
    }

    let config = Config::new();

    let config_path = PathBuf::from(&config.path);
    let root_dir = config_path.parent().unwrap();
    let cache_dir = root_dir.join(&config.cache.path);

    let cache = Cache::new(cache_dir);

    match &cli.command {
        Some(cli::Command::Cache(cache_command)) => match cache_command {
            CacheCommand::Clear => {
                if let Some(profile) = &cli.profile {
                    cache.clear(&profile.clone())
                }
                process::exit(1);
            }
            CacheCommand::Invalidate => {
                if let Some(profile) = &cli.profile {
                    cache.invalidate(&profile.clone())
                }
                process::exit(1);
            }
        },
        None => {}
    }

    let ConfigEvaluation {
        project_id,
        profile_name,
        max_age,
        r#override,
    } = config.evaluate(cli.profile.to_owned()).unwrap();

    let (program, program_args) = match get_program(&cli) {
        Some(t) => t,
        None => {
            log::error!("no slop provided");
            std::process::exit(1)
        }
    };

    let CacheEntry {
        variables: secrets, ..
    } = cache
        .get_or_revalidate(&profile_name, max_age, move || async {
            let mut bitwarden_client = BitwardenClient::new(cli.token).await;
            bitwarden_client.get_secrets_by_project_id(project_id).await
        })
        .await
        .unwrap();

    let mut final_secrets = std::collections::HashMap::new();

    for (key, value) in secrets.into_iter() {
        final_secrets.insert(key, value);
    }

    if let Some(overrides) = &r#override {
        for (key, value) in overrides {
            final_secrets.insert(key.clone(), value.clone());
        }
    }

    let secrets = final_secrets.into_iter().collect::<Vec<(String, String)>>();

    let mut cmd = Command::new(program);

    cmd.args(program_args);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    cmd.envs(secrets.to_owned());

    if let Ok(mut child) = cmd.spawn() {
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let mut buffer = [0; 1024];

        // Create separate threads to handle stdout and stderr
        let stdout_thread = std::thread::spawn(move || loop {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = String::from_utf8(buffer[0..n].to_vec()) {
                        print!("{}", s);
                        io::stdout().flush().expect("Failed to flush stdout");
                    }
                }
                Err(err) => {
                    eprintln!("Error reading child process stdout: {:?}", err);
                    break;
                }
            }
        });

        let stderr_thread = std::thread::spawn(move || loop {
            match stderr.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = String::from_utf8(buffer[0..n].to_vec()) {
                        eprint!("{}", s);
                        io::stderr().flush().expect("Failed to flush stderr");
                    }
                }
                Err(err) => {
                    eprintln!("Error reading child process stderr: {:?}", err);
                    break;
                }
            }
        });

        // Wait for the child process to finish and close the threads
        let status = child.wait().expect("Failed to wait on child process");
        stdout_thread.join().expect("stdout thread panicked");
        stderr_thread.join().expect("stderr thread panicked");

        process::exit(status.code().unwrap_or(1))
    }
}

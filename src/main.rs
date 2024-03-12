#![allow(warnings)]
use clap::Parser;
use cli::CacheCommand;
use log::Level;
use semver::Version;
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Command, Stdio},
};

mod bitwarden;
mod cache;
mod cli;
mod config;
mod config_yaml;
mod error;
mod fs;

use cache::CacheEntry;

use crate::{bitwarden::BitwardenClient, cli::Cli};
use crate::{cache::Cache, config_yaml::find_local_config};

use crate::config_yaml::{Config, ConfigEvaluation};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    pub fn get_program(cli: &Cli) -> Option<(String, Vec<String>)> {
        let slop = &cli.slop;
        match &slop.first() {
            Some(program) => {
                let args = slop[1..].to_vec();

                Some((program.to_string(), args))
            }
            None => None,
        }
    }

    let config_path = find_local_config().unwrap();
    let config = Config::new(&config_path).unwrap();

    let root_dir = config_path.parent().unwrap();
    let cache_dir = root_dir.join(config.cache.path.as_pathbuf());

    let cache = Cache::new(cache_dir);

    let profile_name = cli.profile.clone().unwrap_or(String::from("default"));

    let ConfigEvaluation {
        version_req,
        max_age,
        project_id,
        ..
    } = config.evaluate(&profile_name).unwrap();

    match &cli.command {
        Some(cli::Command::Cache(cache_command)) => match cache_command {
            CacheCommand::Clear => {
                cache.clear(&profile_name);
                process::exit(0);
            }
            CacheCommand::Invalidate => {
                cache.invalidate(&profile_name);
                process::exit(0);
            }
        },
        None => {}
    }

    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    if !version_req.matches(&version) {
        log::error!(
            "Version {} does not meet the requirement {}",
            version,
            version_req
        );
        std::process::exit(1);
    }

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
        .get_or_revalidate(&profile_name, max_age.as_u64(), move || {
            let project_id = project_id.clone();
            async move {
                let mut bitwarden_client = BitwardenClient::new(cli.token).await;
                bitwarden_client
                    .get_secrets_by_project_id(&project_id)
                    .await
            }
        })
        .await
        .unwrap();

    let mut final_secrets = std::collections::HashMap::new();

    for (key, value) in secrets.into_iter() {
        final_secrets.insert(key, value);
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

    Ok(())
}

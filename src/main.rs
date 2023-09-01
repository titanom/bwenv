use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

mod bitwarden;
mod cache;
mod cli;
mod config;

use crate::cache::Cache;
use crate::config::Config;
use crate::{bitwarden::BitwardenClient, cli::Cli};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::new();
    let (program, program_args) = cli.get_program();

    let config = Config::new();
    let (project_id, profile_name) = config.evaluate();

    let cache = Cache::new(PathBuf::from(config.cache.path));

    let cached_env = cache.get(&profile_name);

    let mut cmd = Command::new(program);

    cmd.args(program_args);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let secrets = match cached_env {
        Some(cache_entry) => cache_entry
            .variables
            .into_iter()
            .collect::<Vec<(String, String)>>(),
        None => {
            let mut bitwarden_client = BitwardenClient::new(cli.args.token).await;
            bitwarden_client.get_secrets_by_project_id(project_id).await
        }
    };

    cache.set("development", &secrets);

    cmd.envs(secrets);

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
        let _ = child.wait();
        stdout_thread.join().expect("stdout thread panicked");
        stderr_thread.join().expect("stderr thread panicked");
    }
}

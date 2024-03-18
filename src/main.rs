use clap::Parser;
use cli::CacheCommand;
use semver::Version;
use std::{
    io::{self, Read, Write},
    path::Path,
    process::{self, Command, Stdio},
};

use tracing::{error, info, span, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod bitwarden;
mod cache;
mod cli;
mod config;
mod config_toml;
mod config_yaml;
mod error;
mod fs;
mod time;

use cache::CacheEntry;

use crate::cache::Cache;
use crate::{bitwarden::BitwardenClient, cli::Cli, config_yaml::Secrets};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(EnvFilter::new(cli.log_level.as_tracing_env()))
        .with(
            fmt::layer()
                .fmt_fields(fmt::format::PrettyFields::new())
                .event_format(fmt::format().compact().without_time().with_target(false))
                .with_writer(std::io::stdout),
        )
        .init();

    let root_span = span!(Level::INFO, env!("CARGO_PKG_NAME"));
    let _guard = root_span.enter();

    let local_config = config::find_local_config(Some(&std::env::current_dir().unwrap())).unwrap();

    let config_path = local_config.as_pathbuf();

    match local_config {
        config::LocalConfig::Yaml(_) => {
            let config = config_yaml::Config::new(config_path).unwrap();
            run_with(cli, config_path, config).await
        }
        config::LocalConfig::Toml(_) => {
            let toml_config = config_toml::Config::new(config_path).unwrap();
            let config = toml_config.as_yaml_config();
            warn!("bwenv.toml is deprecated. Please migrate to bwenv.yaml");
            run_with(cli, config_path, config).await
        }
    }?;

    Ok(())
}

async fn run_with<'a>(
    cli: Cli,
    config_path: &Path,
    config: config_yaml::Config<'a>,
) -> anyhow::Result<()> {
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

    let root_dir = config_path.parent().unwrap();
    let cache_dir = root_dir.join(config.cache.path.as_pathbuf());

    let profile_name = cli.profile.clone().unwrap_or_else(|| {
        info!(message = "No profile specified, falling back to default profile");
        String::from("default")
    });

    let config_yaml::ConfigEvaluation {
        version_req,
        max_age,
        project_id,
        overrides,
        ..
    } = config.evaluate(&profile_name).unwrap_or_else(|_| {
        error!(
            message = format!(
                "Could not find configuration for profile {:?}",
                profile_name
            )
        );
        process::exit(1)
    });

    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    let cache = Cache::new(cache_dir, &version);

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
        Some(_) => {}
    }

    if !version_req.matches(&version) {
        error!(
            "Version {} does not meet the requirement {}",
            version, version_req
        );
        std::process::exit(1);
    }

    let token = cli.token.clone();
    let CacheEntry { variables, .. } = cache
        .get_or_revalidate(&profile_name, max_age.as_u64(), move || async move {
            let mut bitwarden_client = BitwardenClient::new(token).await;
            bitwarden_client
                .get_secrets_by_project_id(&project_id)
                .await
        })
        .await
        .unwrap();

    let mut secrets = Secrets::merge(&variables, &overrides);

    if let Some(cli::Command::Inspect(inspect_args)) = &cli.command {
        let reveal = if inspect_args.reveal {
                inquire::Confirm::new("reveal secrets in output")
                    .with_default(false)
                    .with_help_message("Enabling this option will display sensitive information in plain text. Use with caution, especially in shared or public environments.")
                    .prompt()
            } else {
                Ok(false)
            }
            .unwrap();

        print!("{}", &secrets.table(reveal));
        process::exit(1);
    }

    let (program, program_args) = match get_program(&cli) {
        Some(t) => t,
        None => {
            error!("no slop provided");
            std::process::exit(1)
        }
    };

    let mut cmd = Command::new(program);
    cmd.envs(secrets.as_vec());
    cmd.args(program_args);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

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

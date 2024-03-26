use clap::Parser;
use cli::CacheCommand;
use semver::Version;
use std::{
    cmp::Ordering,
    io::{self, Read, Write},
    path::Path,
    process::{self, Command, Stdio},
    time,
};

use tracing::{error, info, span, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod cli;

use bwenv_lib::cache;
use bwenv_lib::config;
use bwenv_lib::config_toml;
use bwenv_lib::config_yaml;
use bwenv_lib::data;
use bwenv_lib::version;
use bwenv_lib::{bitwarden, time::is_date_older_than_n_seconds};

use cache::CacheEntry;

use crate::cache::Cache;
use crate::{bitwarden::BitwardenClient, cli::Cli, config_yaml::Secrets};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

    let version =
        Version::parse(env!("CARGO_PKG_VERSION")).expect("Failed to parse cargo package version");

    let data = data::Data::new();

    let data_content = data.get_content();
    let latest_version: Option<String> =
        if is_date_older_than_n_seconds(data_content.last_update_check, &86400_u64)
            || data_content.last_checked_version.is_none()
        {
            if let Ok(latest_version) = version::fetch_latest_version().await {
                let _ = data.set_content(
                    time::SystemTime::now()
                        .duration_since(time::SystemTime::UNIX_EPOCH)
                        .expect("SystemTime before UNIX EPOCH!")
                        .as_millis()
                        .try_into()
                        .expect("Failed to convert system time"),
                    Version::to_string(&latest_version),
                );
                Some(latest_version.to_string())
            } else {
                error!("Failed to fetch latest version");
                None
            }
        } else {
            data_content.last_checked_version
        };

    if let Some(latest_version) = &latest_version {
        if let Ok(latest_version) = Version::parse(latest_version) {
            let ordering = version.cmp_precedence(&latest_version);
            if ordering == Ordering::Less {
                info!(message = format!("New version available: {}", &latest_version));
            }
        }
    }

    let local_config = config::find_local_config(Some(
        &std::env::current_dir().expect("Failed to retrieve CWD"),
    ))
    .unwrap_or_else(|_| {
        error!("Failed to find local configuration file");
        std::process::exit(1);
    });

    let config_path = local_config.as_pathbuf();

    match local_config {
        config::LocalConfig::Yaml(_) => {
            let config = config_yaml::Config::new(config_path).unwrap();
            run_with(cli, config_path, config, version).await
        }
        config::LocalConfig::Toml(_) => {
            let toml_config = config_toml::Config::new(config_path).unwrap();
            let config = toml_config.as_yaml_config();
            warn!("bwenv.toml is deprecated. Please migrate to bwenv.yaml");
            run_with(cli, config_path, config, version).await
        }
    };
}

async fn run_with<'a>(
    cli: Cli,
    config_path: &Path,
    config: config_yaml::Config<'a>,
    version: Version,
) {
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

    let root_dir = config_path
        .parent()
        .expect("Failed to resolve root directory");
    let cache_dir = root_dir.join(config.cache.path.as_path());

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
        .get_or_revalidate(&profile_name, max_age, move || async move {
            let mut bitwarden_client = BitwardenClient::new(token).await;
            bitwarden_client
                .get_secrets_by_project_id(&project_id)
                .await
                .unwrap()
        })
        .await
        .unwrap();

    let mut secrets = Secrets::merge(&variables, &overrides);

    if let Some(cli::Command::Inspect(inspect_args)) = &cli.command {
        let is_terminal = atty::is(atty::Stream::Stdout);
        let reveal = if inspect_args.reveal && is_terminal {
                inquire::Confirm::new("reveal secrets in output")
                    .with_default(false)
                    .with_help_message("Enabling this option will display sensitive information in plain text. Use with caution, especially in shared or public environments.")
                    .prompt()
            } else {
                Ok(inspect_args.reveal)
            }
            .unwrap();

        print!("{}", &secrets.table(reveal));
        process::exit(0);
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
        let mut stdout = child.stdout.take().expect("Failed to take stdout");
        let mut stderr = child.stderr.take().expect("Failed to take stderr");
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

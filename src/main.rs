use std::env;

use bitwarden::{
    auth::request::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    secrets_manager::secrets::SecretIdentifiersRequest,
    Client,
};
use clap::Parser;

mod cli;
mod config;

use crate::{cli::Args, config::parse_local_config};

fn main() {
    let local_config = parse_local_config().unwrap();

    let _ = evaluate_config(&local_config);

    println!("local_config: {:?}", local_config);
}

fn get_profile_from_env(env_var_names: &Vec<String>) -> Option<String> {
    let mut existing_env_vars = Vec::new();

    for env_var_name in env_var_names {
        if let Ok(env_var_value) = env::var(env_var_name) {
            existing_env_vars.push(env_var_value);
        }
    }

    existing_env_vars.first().map(|s| s.to_string())
}

fn evaluate_config(local_config: &config::Config) {
    let env_var_names = local_config.environment.as_ref().unwrap();
    let env_profile = get_profile_from_env(env_var_names)
        .expect("please provide a profile via environment variables");

    println!("env_profile: {:?}", env_profile);
}

#[tokio::main(flavor = "current_thread")]
async fn not_main() {
    let local_config = parse_local_config().unwrap();

    println!("local_config: {:?}", local_config);

    let args = Args::parse();

    let mut bw_client = Client::new(Some(ClientSettings {
        identity_url: "https://identity.bitwarden.com".to_string(),
        api_url: "https://api.bitwarden.com".to_string(),
        user_agent: "Bitwarden Rust-SDK".to_string(),
        device_type: DeviceType::SDK,
    }));

    bw_client
        .access_token_login(&AccessTokenLoginRequest {
            access_token: args.token,
        })
        .await
        .unwrap();

    let bw_organization = SecretIdentifiersRequest {
        organization_id: bw_client.get_access_token_organization().unwrap(),
    };
    println!(
        "Stored secrets: {:#?}",
        bw_client.secrets().list(&bw_organization).await.unwrap()
    );
}

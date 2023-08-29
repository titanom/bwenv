use bitwarden::{
    auth::request::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    secrets_manager::secrets::SecretIdentifiersRequest,
    Client,
};
use clap::Parser;

mod cli;
mod config;

use crate::cli::Args;
use crate::config::{find_up, parse_config_file};

fn main() {
    let config_file_path = find_up("bwenv.toml", None).unwrap();

    let _ = parse_config_file(&config_file_path);
}

#[tokio::main(flavor = "current_thread")]
async fn not_main() {
    let config_file_path = find_up("bwenv.toml", None).unwrap();

    let _ = parse_config_file(&config_file_path);

    println!("config file: {:?}", config_file_path);

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

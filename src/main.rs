use bitwarden::{
    auth::request::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    secrets_manager::secrets::SecretIdentifiersRequest,
    Client,
};
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

    #[arg(
        short,
        long,
        long_help = "Secret manager project name",
        required = true
    )]
    project: String,

    #[arg(short, long, long_help = "Environment of the project", required = true)]
    environment: String,

    #[arg(
        short,
        long,
        long_help = "Cache directory for the secrets",
        required = true
    )]
    cache_dir: String,

    #[arg(
        short,
        long,
        long_help = "Revalidate the cache after the giben number of seconds",
        default_value_t = 3600
    )]
    revalidate: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

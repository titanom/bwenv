use bitwarden::{
    auth::request::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    secrets_manager::secrets::SecretIdentifiersRequest,
    Client,
};
use clap::Parser;
use serde::Deserialize;
use std::{collections::BTreeMap, env, fs::File, io::Read, path::PathBuf};
use toml::de::Error;

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

#[derive(Debug)]
enum Preset {
    Node,
    Python
}

impl<'de> Deserialize<'de> for Preset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "node" => Ok(Preset::Node),
            "python" => Ok(Preset::Python),
            _ => Err(serde::de::Error::custom(format!("unknown preset: {}", s)))
        }
    }
}

#[derive(Debug, Deserialize)]
struct Environment {
    project: Option<String>,
    environment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
    preset: Option<Preset>,
    project: Option<String>,
    #[serde(flatten)]
    environments: BTreeMap<String, Environment>,
}

fn parse_config_file(file_path: &PathBuf) -> Result<Config, Error> {
    let mut toml_str = String::new();
    let mut file = File::open(file_path).unwrap();
    file.read_to_string(&mut toml_str).unwrap();

    let config: Config = toml::from_str(&toml_str).unwrap();
    println!("config: {:?}", config);
    Ok(config)
}

fn find_up(filename: &str, max_parents: Option<i32>) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut current_path = current_dir.as_path();

    for _ in 0..max_parents.unwrap_or(10) {
        let file_path = current_path.join(filename);

        if file_path.exists() {
            return Some(file_path);
        }

        match current_path.parent() {
            Some(parent) => current_path = parent,
            None => break,
        }
    }

    None
}

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

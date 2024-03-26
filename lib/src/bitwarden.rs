use std::sync::Mutex;

use bitwarden::secrets_manager::secrets::{
    SecretIdentifiersByProjectRequest, SecretIdentifiersResponse, SecretsGetRequest,
};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    Client,
};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;
use uuid::Uuid;

use crate::config_yaml::Secrets;
use tracing::{error, info};

pub struct BitwardenClient {
    _identity_url: String,
    _api_url: String,
    _user_agent: String,
    _device_type: DeviceType,
    _access_token: String,
    client: Mutex<Client>,
}

impl BitwardenClient {
    pub async fn new(access_token: String) -> Self {
        let identity_url = String::from("https://identity.bitwarden.com");
        let api_url = String::from("https://api.bitwarden.com");
        let user_agent = String::from("Bitwarden Rust-SDK");
        let device_type = DeviceType::SDK;

        let mut client = Client::new(Some(ClientSettings {
            identity_url: identity_url.to_owned(),
            api_url: api_url.to_owned(),
            user_agent: user_agent.to_owned(),
            device_type,
        }));

        client
            .access_token_login(&AccessTokenLoginRequest {
                access_token: access_token.to_owned(),
            })
            .await
            .unwrap_or_else(|_| {
                error!(message = "Failed to login using access token");
                std::process::exit(1);
            });

        Self {
            client: Mutex::new(client),
            _access_token: access_token,
            _identity_url: identity_url,
            _api_url: api_url,
            _user_agent: user_agent,
            _device_type: device_type,
        }
    }

    pub async fn get_secrets_by_project_id<'a, T: AsRef<str>>(
        &mut self,
        project_id: T,
    ) -> Result<Secrets<'a>, Box<dyn std::error::Error>> {
        let secret_identifiers = async {
            let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(3);
            let request = || async {
                let secrets_by_project_request = SecretIdentifiersByProjectRequest {
                    project_id: Uuid::parse_str(project_id.as_ref())?,
                };

                info!(message = "Fetching secret IDs");

                Ok(self
                    .client
                    .lock()
                    .unwrap()
                    .secrets()
                    .list_by_project(&secrets_by_project_request)
                    .await?)
            };

            let result: Result<SecretIdentifiersResponse, Box<dyn std::error::Error>> =
                Retry::spawn(retry_strategy, request).await;

            result.unwrap()
        };

        let ids: Vec<Uuid> = secret_identifiers
            .await
            .data
            .into_iter()
            .map(|ident| ident.id)
            .collect();

        let secrets = async {
            let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(3);
            let request = || async {
                let secrets_get_request = SecretsGetRequest { ids: ids.clone() };

                info!(message = "Fetching secrets");

                Ok(self
                    .client
                    .lock()
                    .unwrap()
                    .secrets()
                    .get_by_ids(secrets_get_request)
                    .await?
                    .data
                    .into_iter()
                    .map(|secret| (secret.key, secret.value))
                    .collect())
            };

            let result: Result<Secrets<'a>, Box<dyn std::error::Error>> =
                Retry::spawn(retry_strategy, request).await;

            result
        };

        secrets.await
    }
}

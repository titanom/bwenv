use bitwarden::secrets_manager::secrets::{SecretIdentifiersByProjectRequest, SecretsGetRequest};
use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    client::client_settings::{ClientSettings, DeviceType},
    Client,
};
use uuid::Uuid;

pub struct BitwardenClient {
    _identity_url: String,
    _api_url: String,
    _user_agent: String,
    _device_type: DeviceType,
    _access_token: String,
    client: Client,
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
            .unwrap();

        Self {
            client,
            _access_token: access_token,
            _identity_url: identity_url,
            _api_url: api_url,
            _user_agent: user_agent,
            _device_type: device_type,
        }
    }

    pub async fn get_secrets_by_project_id(&mut self, project_id: String) -> Vec<(String, String)> {
        let secrets_by_project_request = SecretIdentifiersByProjectRequest {
            project_id: Uuid::parse_str(&project_id).unwrap(),
        };

        let secret_identifiers = self
            .client
            .secrets()
            .list_by_project(&secrets_by_project_request)
            .await
            .unwrap();

        let secrets_get_request = SecretsGetRequest {
            ids: secret_identifiers
                .data
                .into_iter()
                .map(|ident| ident.id)
                .collect(),
        };

        self.client
            .secrets()
            .get_by_ids(secrets_get_request)
            .await
            .unwrap()
            .data
            .into_iter()
            .map(|secret| (secret.key, secret.value))
            .collect()
    }
}

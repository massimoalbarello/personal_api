use std::env;

use serde::{Deserialize, Serialize};
use serde_json::Value;

const ACCESS_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const TRANSFER_BASE_URL: &str = "https://dataportability.googleapis.com/v1beta/";
const INITIATE_TRANSFER_ENDPOINT: &str = "portabilityArchive:initiate";
pub struct AccessTokenUrl {
    endpoint: String,
    params: AccessTokenParams,
}

impl AccessTokenUrl {
    pub fn new(params: AccessTokenParams) -> Self {
        Self {
            endpoint: String::from(ACCESS_TOKEN_ENDPOINT),
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenParams {
    state: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    client_id: String,
    client_secret: String,
    grant_type: String,
}

impl AccessTokenParams {
    pub fn default() -> Self {
        Self {
            state: None,
            code: None,
            redirect_uri: None,
            client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),
            grant_type: String::from("authorization_code"),
        }
    }

    pub fn with_state(mut self, state: String) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_redirect_uri(mut self, redirect_uri: String) -> Self {
        self.redirect_uri = Some(redirect_uri);
        self
    }

    pub fn as_url(&self) -> String {
        serde_json::to_value(&self)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .fold(String::new(), |params, (param, value)| match value {
                Value::String(value) => {
                    if !params.is_empty() {
                        return format!("{}&{}={}", params, param, value);
                    }
                    format!("{}={}", param, value)
                }
                _ => params,
            })
    }
}

#[derive(Deserialize, Debug)]
pub struct AccessTokenResponsePayload {
    access_token: String,
    expires_in: u32,
    scope: String,
    token_type: String,
}

impl AccessTokenResponsePayload {
    pub fn access_token(&self) -> String {
        self.access_token.clone()
    }
}

pub struct InitiateTransferUrl {
    endpoint: String,
    params: InitiateTransferParams,
}

impl InitiateTransferUrl {
    pub fn new(params: InitiateTransferParams) -> Self {
        Self {
            endpoint: String::from(TRANSFER_BASE_URL) + INITIATE_TRANSFER_ENDPOINT,
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateTransferParams {
    resources: Option<String>,
    alt: String,
}

impl InitiateTransferParams {
    pub fn default() -> Self {
        Self {
            resources: None,
            alt: String::from("json"),
        }
    }

    pub fn with_resources(mut self, resources: String) -> Self {
        self.resources = Some(resources);
        self
    }

    pub fn as_url(&self) -> String {
        serde_json::to_value(&self)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .fold(String::new(), |params, (param, value)| match value {
                Value::String(value) => {
                    if !params.is_empty() {
                        return format!("{}&{}={}", params, param, value);
                    }
                    format!("{}={}", param, value)
                }
                _ => params,
            })
    }
}

#[derive(Deserialize, Debug)]
pub struct InitiateTransferResponsePayload {
    #[serde(rename = "archiveJobId")]
    archive_job_id: String,
}

impl InitiateTransferResponsePayload {
    pub fn archive_job_id(&self) -> String {
        self.archive_job_id.clone()
    }
}

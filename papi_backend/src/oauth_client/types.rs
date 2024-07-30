use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::env;

const ACCESS_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const ARCHIVE_BASE_URL: &str = "https://dataportability.googleapis.com/v1beta/";
const INITIATE_ARCHIVE_ENDPOINT: &str = "portabilityArchive:initiate";
const ARCHIVE_JOBS_ENDPOINT: &str = "archiveJobs/";
const POLL_ARCHIVE_STATE_ENDPOINT: &str = "/portabilityArchiveState";
const RESET_AUTHORIZATION_ENDPOINT: &str = "authorization:reset";

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

    pub fn expires_in(&self) -> u32 {
        self.expires_in
    }

    pub fn scope(&self) -> String {
        self.scope.clone()
    }
}

pub struct InitiateArchiveUrl {
    endpoint: String,
    params: InitiateArchiveParams,
}

impl InitiateArchiveUrl {
    pub fn new(params: InitiateArchiveParams) -> Self {
        Self {
            endpoint: String::from(ARCHIVE_BASE_URL) + INITIATE_ARCHIVE_ENDPOINT,
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitiateArchiveParams {
    resources: Option<String>,
    alt: String,
}

impl InitiateArchiveParams {
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
pub struct InitiateArchiveResponsePayload {
    #[serde(rename = "archiveJobId")]
    archive_job_id: String,
}

impl InitiateArchiveResponsePayload {
    pub fn archive_job_id(&self) -> String {
        self.archive_job_id.clone()
    }
}

pub struct GetArchiveStateUrl {
    endpoint: String,
    params: GetArchiveStateParams,
}

impl GetArchiveStateUrl {
    pub fn new(job_id: String, params: GetArchiveStateParams) -> Self {
        Self {
            endpoint: format!(
                "{}{}{}{}",
                ARCHIVE_BASE_URL, ARCHIVE_JOBS_ENDPOINT, job_id, POLL_ARCHIVE_STATE_ENDPOINT
            ),
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetArchiveStateParams {
    alt: String,
}

impl GetArchiveStateParams {
    pub fn default() -> Self {
        Self {
            alt: String::from("json"),
        }
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

#[derive(Debug)]
pub enum GetArchiveStateResponsePayload {
    Completed(ArchiveCompleteResponsePayload),
    InProgress(ArchiveInProgressResponsePayload),
    // Failed(ArchiveFailedResponsePayload),    // TODO: figure out how the response looks like in case "state" is "FAILED"
}

impl<'de> Deserialize<'de> for GetArchiveStateResponsePayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Ok(response) = ArchiveCompleteResponsePayload::deserialize(&value) {
            return Ok(GetArchiveStateResponsePayload::Completed(response));
        }
        if let Ok(response) = ArchiveInProgressResponsePayload::deserialize(&value) {
            return Ok(GetArchiveStateResponsePayload::InProgress(response));
        }
        Err(serde::de::Error::custom(
            "Failed to deserialize to existing variants",
        ))
    }
}

#[derive(Deserialize, Debug)]
pub struct ArchiveCompleteResponsePayload {
    state: String,
    urls: Vec<String>,
}

impl ArchiveCompleteResponsePayload {
    pub fn state(&self) -> String {
        self.state.clone()
    }

    pub fn urls(&self) -> Vec<String> {
        self.urls.clone()
    }
}

#[derive(Deserialize, Debug)]
pub struct ArchiveInProgressResponsePayload {
    state: String,
}

impl ArchiveInProgressResponsePayload {
    pub fn state(&self) -> String {
        self.state.clone()
    }
}

pub struct ResetAuthorizationUrl {
    endpoint: String,
    params: ResetAuthorizationParams,
}

impl ResetAuthorizationUrl {
    pub fn new(params: ResetAuthorizationParams) -> Self {
        Self {
            endpoint: format!("{}{}", ARCHIVE_BASE_URL, RESET_AUTHORIZATION_ENDPOINT),
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetAuthorizationParams {
    alt: String,
}

impl ResetAuthorizationParams {
    pub fn default() -> Self {
        Self {
            alt: String::from("json"),
        }
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
pub struct ResetAuthorizationResponsePayload {}

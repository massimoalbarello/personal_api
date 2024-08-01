use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::{collections::HashMap, sync::RwLock};

pub type UserId = String;

pub type OAuthState = String;

pub type OAuthCode = String;

pub type AccessToken = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceState {
    Granted,
    Initiated,
    Downloaded,
}

pub type Resource = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OAuthAccessToken {
    token: AccessToken,
    expires_at: i64,
    granted_resources: HashMap<Resource, ResourceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthInfo {
    user_id: UserId,
    state: OAuthState,
    code: OAuthCode,
    access_token: Option<OAuthAccessToken>,
}

impl OAuthInfo {
    pub fn new(user_id: UserId, state: OAuthState, code: OAuthCode) -> Self {
        Self {
            user_id,
            state,
            code,
            access_token: None,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id.clone()
    }

    pub fn state(&self) -> String {
        self.state.clone()
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }

    pub fn access_token(&self) -> Option<String> {
        self.access_token.as_ref().map(|a| a.token.clone())
    }

    pub fn set_access_token(&mut self, token: AccessToken, expires_in: u32, scope: String) {
        self.access_token = Some(OAuthAccessToken {
            token,
            expires_at: Utc::now().timestamp() + expires_in as i64,
            granted_resources: extract_my_activity_resources(&scope),
        });
    }

    pub fn update_granted_resource(&mut self, resource: &str, new_resource_state: ResourceState) {
        match self.access_token.as_mut() {
            Some(a) => {
                if let Some(resource_state) = a.granted_resources.get_mut(resource) {
                    *resource_state = new_resource_state;
                } else {
                    print!("User did not grant access to resource: {}", resource);
                }
            }
            None => print!("Access token not found"),
        }
    }
}

fn extract_my_activity_resources(scope: &str) -> HashMap<Resource, ResourceState> {
    let re =
        Regex::new(r"https://www.googleapis.com/auth/dataportability\.(myactivity\.\w+)").unwrap();
    let mut results = HashMap::new();

    for cap in re.captures_iter(scope) {
        if let Some(matched) = cap.get(1) {
            results.insert(matched.as_str().to_string(), ResourceState::Granted);
        } else {
            print!("Failed to extract resource from {}", scope);
        }
    }

    results
}

pub type UserStateMap = RwLock<HashMap<UserId, OAuthState>>;

const AUTHORIZATION_BASE_URL: &str =
    "https://accounts.google.com/o/oauth2/v2/auth/oauthchooseaccount";

pub struct AuthorizationUrl {
    endpoint: String,
    params: AuthorizationParams,
}

impl AuthorizationUrl {
    pub fn new(params: AuthorizationParams) -> Self {
        Self {
            endpoint: String::from(AUTHORIZATION_BASE_URL),
            params,
        }
    }

    pub fn as_url(&self) -> String {
        format!("{}?{}", self.endpoint, self.params.as_url())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizationParams {
    state: Option<String>,
    scope: Option<String>,
    redirect_uri: Option<String>,
    client_id: String,
    access_type: String,
    include_granted_scopes: bool,
    response_type: String,
    service: String,
    o2v: String,
    ddms: String,
    flow_name: String,
}

impl AuthorizationParams {
    pub fn default() -> Self {
        Self {
            state: None,
            scope: None,
            redirect_uri: None,
            client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            access_type: String::from("offline"),
            include_granted_scopes: true,
            response_type: String::from("code"),
            service: String::from("lso"),
            o2v: String::from("2"),
            ddms: String::from("0"),
            flow_name: String::from("GenerateOAuthFlow"),
        }
    }

    pub fn with_state(mut self, state: String) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
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
                Value::Bool(bool) => format!("{}&{}={}", params, param, bool),
                _ => params,
            })
    }
}

#[derive(Deserialize)]
pub struct AuthorizationCodeRequestPayload {
    state: String,
    code: String,
}

impl AuthorizationCodeRequestPayload {
    pub fn state(&self) -> String {
        self.state.clone()
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }
}

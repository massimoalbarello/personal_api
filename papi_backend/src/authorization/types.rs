use crate::RESOURCES;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;

pub type Authorizations = RwLock<HashMap<String, AuthorizationState>>;

const AUTHORIZATION_BASE_URL: &str =
    "https://accounts.google.com/o/oauth2/v2/auth/oauthchooseaccount";

#[derive(Debug)]
pub struct AuthorizationState {
    state: String,
    code: Option<String>,
    access_token: Option<String>,
    resource_res: Vec<(String, Result<String, String>)>,
}

impl AuthorizationState {
    pub fn new() -> Self {
        let state = Uuid::new_v4().to_string();
        Self {
            state,
            code: None,
            access_token: None,
            resource_res: Vec::new(),
        }
    }

    pub fn state(&self) -> String {
        self.state.clone()
    }

    pub fn code(&self) -> Option<String> {
        self.code.clone()
    }

    pub fn access_token(&self) -> Option<String> {
        self.access_token.clone()
    }

    pub fn set_code(&mut self, code: String) {
        self.code = Some(code);
    }

    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = Some(access_token);
    }

    pub fn push_resource(&mut self, resource: String, res: Result<String, String>) {
        self.resource_res.push((resource, res));
    }

    pub fn all_resources_processed(&self) -> bool {
        self.resource_res.len() == RESOURCES.len()
    }
}

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
    id: String,
    state: String,
    code: String,
}

impl AuthorizationCodeRequestPayload {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn state(&self) -> String {
        self.state.clone()
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }
}

use std::{collections::HashMap, sync::RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type Authorizations = RwLock<HashMap<String, AuthorizationState>>;

pub struct AuthorizationState {
    state: String,
    code: Option<String>,
    access_token: Option<String>,
}

impl AuthorizationState {
    pub fn new() -> Self {
        let state = Uuid::new_v4().to_string();
        Self {
            state,
            code: None,
            access_token: None,
        }
    }

    pub fn state(&self) -> String {
        self.state.clone()
    }
}

pub struct AuthorizationUrl {
    endpoint: String,
    params: AuthorizationParams,
}

impl AuthorizationUrl {
    pub fn new(params: AuthorizationParams) -> Self {
        Self {
            endpoint: String::from(
                "https://accounts.google.com/o/oauth2/v2/auth/oauthchooseaccount",
            ),
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
    client_id: Option<String>,
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
            client_id: None,
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

    pub fn with_client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
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

use std::{collections::HashMap, sync::RwLock};

use actix_web::web::Data;
use reqwest::Client;
use types::{AccessTokenParams, AccessTokenResponsePayload};

use crate::{authorization::types::AuthorizationState, REDIRECT_URI};

mod types;

pub async fn convert_auth_code_to_access_token(
    client_id: String,
    authorizations: Data<RwLock<HashMap<String, AuthorizationState>>>,
) -> Result<String, String> {
    println!(
        "Converting auth code to access token for client ID: {}",
        client_id
    );

    let auth_code = authorizations
        .read()
        .unwrap()
        .get(&client_id)
        .unwrap()
        .code()
        .unwrap();

    let state = authorizations
        .read()
        .unwrap()
        .get(&client_id)
        .unwrap()
        .state()
        .to_string();

    let params = AccessTokenParams::default()
        .with_code(auth_code)
        .with_redirect_uri(REDIRECT_URI.to_string())
        .with_state(state);
    let access_token_url = types::AccessTokenUrl::new(params).as_url();

    println!("Access token URL: {:?}", access_token_url);

    let response = Client::new()
        .post(access_token_url)
        .header("Content-Length", 0) // otherwise the server returns 411
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        return Err(response.text().await.unwrap());
    }
    let response: AccessTokenResponsePayload = response.json().await.unwrap();

    let access_token = response.access_token();

    println!("Access token: {:?}", access_token);

    Ok(access_token)
}

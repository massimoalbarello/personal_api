use actix_web::{
    web::{Data, Json},
    HttpRequest,
};
use mongodb::Client;
use std::env;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::types::{AuthorizationCodeRequestPayload, OAuthInfo, UserStateMap};
use crate::{
    api::{
        api::DATA_PORTABILITY_BASE_URL,
        types::{AuthorizationParams, AuthorizationUrl},
    },
    REQUESTED_RESOURCES,
};

fn get_client_id(req: HttpRequest) -> Result<String, String> {
    let client_id = req
        .headers()
        .get("X-Client-Id")
        .ok_or("Missing X-Client-Id header")?
        .to_str()
        .map_err(|e| format!("Invalid X-Client-Id header: {}", e))?
        .to_string();

    Ok(client_id)
}

pub fn get_google_oauth_url(
    req: HttpRequest,
    auth: Data<UserStateMap>,
    // auth_db_client: Data<Client>,
) -> Result<String, String> {
    let client_id = get_client_id(req)?;

    let oauth_state = Uuid::new_v4().to_string();

    let params = AuthorizationParams::default()
        .with_state(oauth_state.clone())
        .with_scope(
            REQUESTED_RESOURCES
                .map(|r| format!("{}{}", DATA_PORTABILITY_BASE_URL, r))
                .join("+"),
        )
        .with_redirect_uri(env::var("REDIRECT_URI").map_err(|_| "REDIRECT_URI must be set")?);

    let auth_url = AuthorizationUrl::new(params).as_url();

    println!(
        "User with ID: {} requested authorization URL: {}",
        client_id, auth_url
    );

    auth.write()
        .map_err(|e| format!("Lock is poisoned: {}", e))?
        .insert(client_id, oauth_state);

    Ok(auth_url)
}

pub async fn post_google_authorization_code(
    req: HttpRequest,
    payload: Json<AuthorizationCodeRequestPayload>,
    auth: Data<UserStateMap>,
    authorization_tx: Data<UnboundedSender<OAuthInfo>>,
) -> Result<(), String> {
    let client_id = get_client_id(req)?;

    if let Some(oauth_state) = auth
        .write()
        .map_err(|e| format!("Lock is poisoned: {}", e))?
        .remove(&client_id)
    {
        if oauth_state != payload.state() {
            return Err(format!(
                "User with ID: {} sent invalid state. Expected: {}, got: {}",
                client_id,
                oauth_state,
                payload.state()
            ));
        }

        let oauth_code = payload.code();

        println!(
            "User with ID: {} posted authorization code: {}",
            client_id, oauth_code
        );

        let oauth_info = OAuthInfo::new(client_id, oauth_state, oauth_code);

        authorization_tx.send(oauth_info).unwrap();

        return Ok(());
    }

    Err(format!("User with ID: {} not found", client_id))
}

use super::types::{AuthorizationCodeRequestPayload, OAuthInfo, ResourceState, UserStateMap};
use crate::{
    api::{
        api::DATA_PORTABILITY_BASE_URL,
        types::{AuthorizationParams, AuthorizationUrl},
    },
    auth_db_client::{self, AuthDbClient},
    oauth_client::OAuthClient,
    papi_line_client::PapiLineClient,
    REQUESTED_RESOURCES,
};
use actix_web::{
    web::{Data, Json},
    HttpRequest,
};
use std::env;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

fn get_user_id(req: HttpRequest) -> Result<String, String> {
    let user_id = req
        .headers()
        .get("X-Client-Id")
        .ok_or("Missing X-Client-Id header")?
        .to_str()
        .map_err(|e| format!("Invalid X-Client-Id header: {}", e))?
        .to_string();

    Ok(user_id)
}

pub fn get_google_oauth_url(req: HttpRequest, auth: Data<UserStateMap>) -> Result<String, String> {
    let user_id = get_user_id(req)?;

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
        user_id, auth_url
    );

    auth.write()
        .map_err(|e| format!("Lock is poisoned: {}", e))?
        .insert(user_id, oauth_state);

    Ok(auth_url)
}

pub async fn post_google_authorization_code(
    req: HttpRequest,
    payload: Json<AuthorizationCodeRequestPayload>,
    auth: Data<UserStateMap>,
    authorization_tx: Data<UnboundedSender<OAuthInfo>>,
) -> Result<(), String> {
    let user_id = get_user_id(req)?;

    if let Some(oauth_state) = auth
        .write()
        .map_err(|e| format!("Lock is poisoned: {}", e))?
        .remove(&user_id)
    {
        if oauth_state != payload.state() {
            return Err(format!(
                "User with ID: {} sent invalid state. Expected: {}, got: {}",
                user_id,
                oauth_state,
                payload.state()
            ));
        }

        let oauth_code = payload.code();

        println!(
            "User with ID: {} posted authorization code: {}",
            user_id, oauth_code
        );

        let oauth_info = OAuthInfo::new(user_id, oauth_state, oauth_code);

        authorization_tx.send(oauth_info).unwrap();

        return Ok(());
    }

    Err(format!("User with ID: {} not found", user_id))
}

pub async fn handle_data_archive(
    auth_db_client: &AuthDbClient,
    oauth_client: &OAuthClient,
    mut oauth_info: OAuthInfo,
) -> Result<(), String> {
    // convert authorization code to access token
    oauth_client
        .convert_authorization_to_access_token(&mut oauth_info)
        .await
        .map_err(|e| {
            format!(
                "could not convert authorization code to access token: {}",
                e
            )
        })?;

    oauth_client
        .initiate_data_archives(&mut oauth_info)
        .map_err(|e| format!("could not initialize data archives: {}", e))?;

    print!("OAuth info pre store: {:?}", oauth_info);
    auth_db_client
        .create_auth(oauth_info)
        .await
        .map_err(|e| format!("could not store oauth info: {}", e))?;

    Ok(())
}

pub async fn handle_data_download(
    auth_db_client: &AuthDbClient,
    papi_line_client: &PapiLineClient,
    oauth_client: &OAuthClient,
    user_id: String,
    ready_to_download_resource: String,
    resource_res: Result<String, String>,
) -> Result<(), String> {
    let download_url = resource_res.map_err(|e| format!("could not get download URL: {:?}", e))?;
    let mut oauth_info = auth_db_client
        .read_last_auth_for_user(user_id.clone())
        .await
        .map_err(|e| format!("could not read last auth for user: {:?}", e))?;

    if oauth_info
        .is_not_expired_access_token()
        .is_some_and(|b| b == true)
        && oauth_info
            .is_expected_resource_state(&ready_to_download_resource, &ResourceState::Initiated)
            .is_ok_and(|b| b == true)
    {
        let _ = papi_line_client
            .download_file(user_id.clone(), &ready_to_download_resource, &download_url)
            .await
            .map_err(|e| format!("could not download file: {:?}", e))?;
        oauth_info
            .update_granted_resource_state(&ready_to_download_resource, ResourceState::Downloaded)
            .map_err(|e| format!("could not update resource state: {:?}", e))?;
        if oauth_info.is_all_resources_downloaded() {
            println!(
                "All resources downloaded, resetting authorization for user {}",
                user_id
            );
            oauth_client
                .reset_authorization(&mut oauth_info)
                .await
                .map_err(|e| format!("could not reset authorization: {:?}", e))?;
        }
        auth_db_client
            .update_auth_for_user(user_id, oauth_info)
            .await
            .map_err(|e| format!("could not store updated OAuth info: {:?}", e))?;
    } else {
        return Err(format!(
            "Latest access token expired or resource not in expected state: {:?}",
            oauth_info
        ));
    }

    Ok(())
}

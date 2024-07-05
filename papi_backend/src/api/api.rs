use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use mongodb::Client;
use tokio::sync::mpsc::UnboundedSender;

use crate::api::types::{AuthorizationCodeRequestPayload, UserStateMap};

use super::{
    handlers::{get_google_oauth_url, post_google_authorization_code},
    types::OAuthInfo,
};

pub const DATA_PORTABILITY_BASE_URL: &str = "https://www.googleapis.com/auth/dataportability.";

pub async fn get_auth_api(
    req: HttpRequest,
    auth: Data<UserStateMap>,
    auth_db_client: Data<Client>,
) -> impl Responder {
    match get_google_oauth_url(req, auth, auth_db_client) {
        Ok(auth_url) => HttpResponse::Ok()
            .content_type("application/json")
            .json(serde_json::json!({"url": auth_url})),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn post_auth_api(
    req: HttpRequest,
    payload: Json<AuthorizationCodeRequestPayload>,
    auth: Data<UserStateMap>,
    authorization_tx: Data<UnboundedSender<OAuthInfo>>,
) -> impl Responder {
    match post_google_authorization_code(req, payload, auth, authorization_tx).await {
        Ok(()) => HttpResponse::Ok().body("OK"),
        Err(err) => HttpResponse::InternalServerError().body(err),
    }
}

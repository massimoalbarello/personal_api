use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    authorization::types::{
        AuthorizationCodeRequestPayload, AuthorizationParams, AuthorizationState, AuthorizationUrl,
        Authorizations,
    },
    REDIRECT_URI, RESOURCES,
};

const DATA_PORTABILITY_BASE_URL: &str = "https://www.googleapis.com/auth/dataportability.";

pub async fn get_google_oauth_url(req: HttpRequest, auth: Data<Authorizations>) -> impl Responder {
    let client_id = req
        .headers()
        .get("X-Client-Id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let auth_state = AuthorizationState::new();

    let params = AuthorizationParams::default()
        .with_state(auth_state.state())
        .with_scope(
            RESOURCES
                .map(|r| format!("{}{}", DATA_PORTABILITY_BASE_URL, r))
                .join("+"),
        )
        .with_redirect_uri(REDIRECT_URI.to_string());
    let auth_url = AuthorizationUrl::new(params).as_url();

    println!(
        "Client with ID: {} requested authorization URL: {}",
        client_id, auth_url
    );

    auth.write().unwrap().insert(client_id, auth_state);

    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({"url": auth_url}))
}

pub async fn post_google_authorization_code(
    payload: Json<AuthorizationCodeRequestPayload>,
    auth: Data<Authorizations>,
    authorization_tx: Data<UnboundedSender<String>>,
) -> impl Responder {
    let id = payload.id();
    if let Some(auth_state) = auth.write().unwrap().get_mut(&id) {
        if auth_state.state() != payload.state() {
            println!(
                "Client with ID: {} sent invalid state. Expected: {}, got: {}",
                id,
                auth_state.state(),
                payload.state()
            );
            return HttpResponse::BadRequest();
        }

        let auth_code = payload.code();

        println!(
            "Client with ID: {} posted authorization code: {}",
            id, auth_code
        );
        auth_state.set_code(auth_code);

        authorization_tx.send(id).unwrap();

        return HttpResponse::Ok();
    }

    println!("Client with ID: {} not found", id);
    HttpResponse::BadRequest()
}

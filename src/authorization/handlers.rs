use actix_web::{web::Data, HttpResponse, Responder};
use std::env;

use crate::authorization::types::{
    AuthorizationParams, AuthorizationState, AuthorizationUrl, Authorizations,
};

const RESOURCES: [&str; 3] = ["myactivity.search", "myactivity.maps", "myactivity.youtube"];
const DATA_PORTABILITY_BASE_URL: &str = "https://www.googleapis.com/auth/dataportability.";
const REDIRECT_URI: &str = "http://localhost:3000/callback";

pub async fn get_google_oauth_url(id: String, auth: Data<Authorizations>) -> impl Responder {
    let auth_state = AuthorizationState::new();

    let params = AuthorizationParams::default()
        .with_state(auth_state.state())
        .with_client_id(env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"))
        .with_scope(
            RESOURCES
                .map(|r| format!("{}{}", DATA_PORTABILITY_BASE_URL, r))
                .join("+"),
        )
        .with_redirect_uri(REDIRECT_URI.to_string());
    let auth_url = AuthorizationUrl::new(params).as_url();

    auth.write().unwrap().insert(id, auth_state);

    println!("Authorization URL: {}", auth_url);

    HttpResponse::Ok().body(format!(
        "Client can start authorization flow at: {}",
        auth_url
    ))
}

pub async fn post_google_authorization_code(
    auth_code: String,
    auth: Data<Authorizations>,
) -> impl Responder {
    // TODO
    HttpResponse::Ok()
}

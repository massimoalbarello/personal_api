use actix_web::web;
use handlers::{get_google_oauth_url, post_google_authorization_code};

mod handlers;
pub mod types;

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("", web::get().to(get_google_oauth_url))
            .route("", web::post().to(post_google_authorization_code)),
    );
}

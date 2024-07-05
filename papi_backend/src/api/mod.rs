use actix_web::web;
use api::{get_auth_api, post_auth_api};

mod api;
mod handlers;
pub mod types;

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("", web::get().to(get_auth_api))
            .route("", web::post().to(post_auth_api)),
    );
}

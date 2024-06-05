use actix_cors::Cors;
use authorization::{auth_config, types::Authorizations};
use oauth_client::OAuthClient;
use papi_line_client::PapiLineClient;

use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

mod authorization;
mod oauth_client;
mod papi_line_client;

const REDIRECT_URI: &str = "http://localhost:3000/callback";
const RESOURCES: [&str; 3] = ["myactivity.search", "myactivity.maps", "myactivity.youtube"];

#[actix_web::main]
async fn main() {
    dotenv().ok();

    // app state initialized inside the closure passed to HttpServer::new is local to the worker thread and may become de-synced if modified
    // to achieve globally shared state, it must be created outside of the closure passed to HttpServer::new and moved/cloned in
    let authorizations = Data::new(Authorizations::default());

    let (authorization_tx, mut authorization_rx): (
        UnboundedSender<String>,
        UnboundedReceiver<String>,
    ) = tokio::sync::mpsc::unbounded_channel();
    let authorization_tx = Data::new(authorization_tx);

    let (download_info_tx, mut download_info_rx): (
        UnboundedSender<((String, String), Result<String, String>)>,
        UnboundedReceiver<((String, String), Result<String, String>)>,
    ) = tokio::sync::mpsc::unbounded_channel();

    let oauth_client = OAuthClient::new(Data::clone(&authorizations), download_info_tx);
    let papi_line_client = PapiLineClient::new();

    let authorizations_cl = Data::clone(&authorizations);
    tokio::spawn(async move {
        // Start a number of HTTP workers equal to the number of physical CPUs in the system
        HttpServer::new(move || {
            App::new()
                .wrap(
                    Cors::default()
                        .allowed_origin("http://localhost:3000")
                        .allowed_methods(vec!["GET", "POST"])
                        .allow_any_header(),
                )
                .app_data(Data::clone(&authorizations_cl))
                .app_data(Data::clone(&authorization_tx))
                .configure(auth_config)
        })
        .bind(("127.0.0.1", 8080))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    loop {
        select! {
            Some(id) = authorization_rx.recv() => {
                // convert authorization code to access token
                if let Ok(()) = oauth_client.convert_authorization_to_access_token(id.clone()).await {
                    println!("Successfully converted authorization code to access token for client ID: {}", id);
                    oauth_client.initiate_data_archives(id.clone());
                }
            },
            Some(((id, resource), resource_res)) = download_info_rx.recv() => {
                if let Ok(download_url) = &resource_res {
                    papi_line_client.post_download_urls(&id, &resource, download_url).await;
                }
                authorizations.write().unwrap().get_mut(&id).unwrap().push_resource(resource, resource_res);
                if authorizations.read().unwrap().get(&id).unwrap().all_resources_processed() {
                    let _ = oauth_client.reset_authorization(id).await;
                }
            }
        }
    }
}
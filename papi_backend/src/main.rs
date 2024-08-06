use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use api::{
    auth_config,
    handlers::{handle_data_archive, handle_data_download},
    types::{OAuthInfo, UserStateMap},
};
use auth_db_client::AuthDbClient;
use dotenv::dotenv;
use oauth_client::OAuthClient;
use papi_line_client::PapiLineClient;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{env, fs::File, io::BufReader};
use tokio::{
    select, signal,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

mod api;
mod auth_db_client;
mod oauth_client;
mod papi_line_client;

const REQUESTED_RESOURCES: [&str; 2] = ["myactivity.search", "myactivity.shopping"];

fn load_certs() -> Result<ServerConfig, String> {
    let cert_file = &mut BufReader::new(
        File::open(env::var("CERT_FILE_PATH").map_err(|_| "CERT_FILE_PATH must be set")?)
            .map_err(|e| e.to_string())?,
    );
    let key_file = &mut BufReader::new(
        File::open(env::var("KEY_FILE_PATH").map_err(|_| "KEY_FILE_PATH must be set")?)
            .map_err(|e| e.to_string())?,
    );

    let cert_chain = certs(cert_file)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        return Err("Could not locate PKCS 8 private keys.".to_string());
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();
    config
        .with_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| e.to_string())
}

#[actix_web::main]
async fn main() -> Result<(), String> {
    dotenv().ok();

    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    // these are stored in the root cargo directory as "key.pem" and "cert.pem"
    let tls_config = load_certs()?;

    // app state initialized inside the closure passed to HttpServer::new is local to the worker thread and may become de-synced if modified
    // to achieve globally shared state, it must be created outside of the closure passed to HttpServer::new and moved/cloned in
    let authorizations = Data::new(UserStateMap::default());

    let (authorization_tx, mut authorization_rx): (
        UnboundedSender<OAuthInfo>,
        UnboundedReceiver<OAuthInfo>,
    ) = tokio::sync::mpsc::unbounded_channel();
    let authorization_tx = Data::new(authorization_tx);

    let (download_info_tx, mut download_info_rx): (
        UnboundedSender<(String, String, Result<String, String>)>,
        UnboundedReceiver<(String, String, Result<String, String>)>,
    ) = tokio::sync::mpsc::unbounded_channel();

    let oauth_client = OAuthClient::new(download_info_tx);
    let papi_line_client = PapiLineClient::new();
    let auth_db_client = AuthDbClient::setup().await?;

    let authorizations_cl = Data::clone(&authorizations);
    tokio::spawn(async move {
        println!("Starting server...");
        // Start a number of HTTP workers equal to the number of physical CPUs in the system
        HttpServer::new(move || {
            App::new()
                .wrap(
                    Cors::default()
                        // TODO: limit origin
                        .allow_any_origin()
                        .allowed_methods(vec!["GET", "POST"])
                        .allow_any_header(),
                )
                .app_data(Data::clone(&authorizations_cl))
                .app_data(Data::clone(&authorization_tx))
                .configure(auth_config)
        })
        .bind_rustls(("0.0.0.0", 8443), tls_config)
        .unwrap()
        // TODO: remove in production
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    loop {
        select! {
            Some(oauth_info) = authorization_rx.recv() => {
                if let Err(e) = handle_data_archive(&auth_db_client, &oauth_client, oauth_info).await {
                    println!("Error handling data archive: {:?}", e);
                }
            },
            Some((user_id, ready_to_download_resource, resource_res)) = download_info_rx.recv() => {
                if let Err(e) = handle_data_download(
                    &auth_db_client,
                    &papi_line_client,
                    &oauth_client,
                    user_id,
                    ready_to_download_resource,
                    resource_res).await {
                        println!("Error handling data download: {:?}", e);
                    }
            },
            _ = signal::ctrl_c() => {
                println!("Shutting down server...");
                break;
            }
        }
    }

    Ok(())
}

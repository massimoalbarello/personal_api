use std::{env, fs::File, io::BufReader};

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

const RESOURCES: [&str; 3] = ["myactivity.search", "myactivity.maps", "myactivity.youtube"];

#[actix_web::main]
async fn main() {
    dotenv().ok();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut certs_file = BufReader::new(
        File::open(env::var("CERT_FILE_PATH").expect("CERT_FILE_PATH must be set")).unwrap(),
    );
    let mut key_file = BufReader::new(
        File::open(env::var("KEY_FILE_PATH").expect("KEY_FILE_PATH must be set")).unwrap(),
    );

    // load TLS certs and key
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

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
                        // TODO: limit origin
                        .allow_any_origin()
                        .allowed_methods(vec!["GET", "POST"])
                        .allow_any_header(),
                )
                .app_data(Data::clone(&authorizations_cl))
                .app_data(Data::clone(&authorization_tx))
                .configure(auth_config)
        })
        .bind(("0.0.0.0", 8080))
        .unwrap()
        .bind_rustls_0_23(("0.0.0.0", 443), tls_config)
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
                    // wait 30 seconds for dowloading the data before resetting the authorization
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    let _ = oauth_client.reset_authorization(id).await;
                }
            }
        }
    }
}

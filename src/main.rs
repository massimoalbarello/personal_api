use authorization::{auth_config, types::Authorizations};
use oauth_client::OAuthClient;

use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

mod authorization;
mod oauth_client;

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

    let oauth_client = OAuthClient::new(Data::clone(&authorizations));

    let authorizations_cl = Data::clone(&authorizations);
    tokio::spawn(async move {
        // Start a number of HTTP workers equal to the number of physical CPUs in the system
        HttpServer::new(move || {
            App::new()
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
                    oauth_client.get_data_transfer_urls(id).await;
                }

            }
        }
    }
}

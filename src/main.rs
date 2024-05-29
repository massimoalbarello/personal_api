use authorization::{auth_config, types::Authorizations};
use utils::convert_auth_code_to_access_token;

use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

mod authorization;
mod utils;

const REDIRECT_URI: &str = "http://localhost:3000/callback";

#[actix_web::main]
async fn main() {
    dotenv().ok();

    // app state initialized inside the closure passed to HttpServer::new is local to the worker thread and may become de-synced if modified
    // to achieve globally shared state, it must be created outside of the closure passed to HttpServer::new and moved/cloned in
    let authorizations = Data::new(Authorizations::default());

    let (auth_code_tx, mut auth_code_rx): (UnboundedSender<String>, UnboundedReceiver<String>) =
        tokio::sync::mpsc::unbounded_channel();
    let auth_code_tx = Data::new(auth_code_tx);

    let authorizations_cl = Data::clone(&authorizations);
    tokio::spawn(async move {
        // Start a number of HTTP workers equal to the number of physical CPUs in the system
        HttpServer::new(move || {
            App::new()
                .app_data(Data::clone(&authorizations_cl))
                .app_data(Data::clone(&auth_code_tx))
                .configure(auth_config)
        })
        .bind(("127.0.0.1", 8080))
        .unwrap()
        .run()
        .await
        .unwrap();
    });

    loop {
        let authorizations_cl = Data::clone(&authorizations);
        select! {
            Some(id) = auth_code_rx.recv() => {
                // convert authorization code to access token
                if let Ok(access_token) = convert_auth_code_to_access_token(id.clone(), authorizations_cl).await {
                    authorizations
                        .write()
                        .unwrap()
                        .get_mut(&id)
                        .unwrap()
                        .set_access_token(access_token);
                }

                println!("Authrization for client ID {}: {:?}", id, authorizations.read().unwrap().get(&id));
            }
        }
    }
}

use crate::authorization::{auth_config, types::Authorizations};

use actix_web::{web::Data, App, HttpServer};
use dotenv::dotenv;

mod authorization;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // app state initialized inside the closure passed to HttpServer::new is local to the worker thread and may become de-synced if modified
    // to achieve globally shared state, it must be created outside of the closure passed to HttpServer::new and moved/cloned in
    let counter = Data::new(Authorizations::default());

    // Start a number of HTTP workers equal to the number of physical CPUs in the system
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&counter))
            .configure(auth_config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

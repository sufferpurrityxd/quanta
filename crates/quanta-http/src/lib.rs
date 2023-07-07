#![allow(dead_code)]
mod state;

use {
    crate::state::HttpServerState,
    actix_web::{web, App, HttpServer},
    quanta_database::Database,
    std::{net, sync::Arc},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got error when binding addrs to [HttpServer]")]
    BindError(std::io::Error),
    #[error("Got error when trying to HttpServer::run() command: {0}")]
    /// Error whill occur when trying to run new [HttpServer]
    RunServer(std::io::Error),
}
/// Create and run new [`HttpServer`] with all specified api-handlers
pub async fn run_http_server<A: net::ToSocketAddrs>(
    addrs: A,
    database: Arc<Database>,
) -> Result<(), Error> {
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(HttpServerState::new(Arc::clone(&database))))
    })
    .bind(addrs)
    .map_err(Error::BindError)?
    .run()
    .await
    .map_err(Error::RunServer)
}

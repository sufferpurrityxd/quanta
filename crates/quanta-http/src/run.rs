use std::{net, sync::Arc};

use actix_web::{middleware::Logger, web, App, HttpServer};
use quanta_database::Database;

use crate::{routes::api_routes, state::HttpServerState};

#[derive(thiserror::Error, Debug)]
pub enum RunError {
    #[error("Got error when binding addrs to [HttpServer]")]
    Bind(std::io::Error),
    #[error("Got error when trying to HttpServer::run() command: {0}")]
    /// Error whill occur when trying to run new [HttpServer]
    RunServer(std::io::Error),
}
/// Create and run new [`HttpServer`] with all specified api-handlers
pub async fn run_http_server<A: net::ToSocketAddrs>(
    addrs: A,
    database: Arc<Database>,
) -> Result<(), RunError> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(HttpServerState::new(Arc::clone(&database))))
            .configure(api_routes)
    })
    .bind(addrs)
    .map_err(RunError::Bind)?
    .run()
    .await
    .map_err(RunError::RunServer)
}

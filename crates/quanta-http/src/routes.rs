use actix_web::web::{get, scope, ServiceConfig};

use crate::http::{connection::get_connections_list, magnet::get_magnet_links_list};

/// Initialize all Quanta HTTP-API Handler-Routes
pub fn api_routes(application_config: &mut ServiceConfig) {
    application_config.service(
        scope("/api").service(
            scope("/v1")
                .service(scope("/connection").route("/list", get().to(get_connections_list)))
                .service(scope("/magnet").route("/list", get().to(get_magnet_links_list))),
        ),
    );
}

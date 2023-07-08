use actix_web::web::{get, scope, ServiceConfig};

/// Initialize all Quanta HTTP-API Handler-Routes
pub fn api_routes(application_config: &mut ServiceConfig) {
    application_config.service(
        scope("/api").service(scope("/v1").service(scope("magnet").route(
            "/list",
            get().to(crate::http::magnet::get_magnet_links_list),
        ))),
    );
}

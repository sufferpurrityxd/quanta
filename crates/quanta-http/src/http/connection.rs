use actix_web::{web, HttpResponse};

use crate::{http::error::QuantaHttpResponse, state::HttpServerState};

/// Returns HashMap of connections that we are receive from proxyservice
pub async fn get_connections_list(state: web::Data<HttpServerState>) -> QuantaHttpResponse {
    Ok(HttpResponse::Ok().json(
        state
            .network_proxy()
            .get_connections()?,
    ))
}

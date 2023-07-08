use actix_web::HttpResponse;

use crate::http::{error::QuantaHttpResponse, util::StatusResponse};

pub async fn index() -> QuantaHttpResponse {
    Ok(HttpResponse::Ok().json(StatusResponse { status: "Ok" }))
}

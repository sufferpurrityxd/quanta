use actix_web::HttpResponse;

use crate::http::error::QuantaHttpResponse;

/// Status Respose is a simple way for build json-response that looks like
///
/// {
///     "status": "ok"
/// }
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatusResponse<'a> {
    pub status: &'a str,
}
/// Error Respose is a simple way for build json-response that looks like
///
/// {
///     "error": "why"
/// }
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse<'a> {
    pub error: &'a str,
}

pub fn generate_error_response(error: &str) -> QuantaHttpResponse {
    Ok(HttpResponse::BadRequest().json(ErrorResponse { error }))
}

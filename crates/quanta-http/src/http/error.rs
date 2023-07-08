use std::num::ParseIntError;

use actix_web::{body::BoxBody, http::header::ToStrError, HttpResponse, ResponseError};
use quanta_database::DatabaseError;
use quanta_network::ProxyError;

/// Custom HTTP Response that used in api-handlers
pub type QuantaHttpResponse = Result<HttpResponse, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got Internal Server Error")]
    InternalServerError,
}

impl From<DatabaseError> for Error {
    fn from(_: DatabaseError) -> Self { Error::InternalServerError }
}

impl From<ProxyError> for Error {
    fn from(_: ProxyError) -> Self { Error::InternalServerError }
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self { Error::InternalServerError }
}

impl From<ToStrError> for Error {
    fn from(_: ToStrError) -> Self { Error::InternalServerError }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self { Error::InternalServerError }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Error::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            },
        }
    }
}

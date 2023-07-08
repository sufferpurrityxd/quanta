use actix_web::{body::BoxBody, HttpResponse, ResponseError};
use quanta_database::DatabaseError;

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

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Error::InternalServerError => {
                HttpResponse::InternalServerError().json("Internal Server Error")
            },
        }
    }
}

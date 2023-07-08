use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{http::error::QuantaHttpResponse, state::HttpServerState};

/// HTTP-API Response-item that used in [get_magnet_links_list] handler
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MagnetLinkListResponse {
    /// Index of magnet link
    pub id: u64,
    /// String representation of [MagnetLink]
    pub magnet: String,
}
/// Return all correct magnets links that stored in database.
pub async fn get_magnet_links_list(state: web::Data<HttpServerState>) -> QuantaHttpResponse {
    let database = state.database().get_magnet_links()?;
    Ok(HttpResponse::Ok().json(
        database
            .iter()
            .map(|(id, magnet)| MagnetLinkListResponse {
                id: *id,
                magnet: magnet.to_string(),
            })
            .collect::<Vec<MagnetLinkListResponse>>(),
    ))
}

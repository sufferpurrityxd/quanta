use actix_multipart::Multipart;
use actix_web::{web::Data, HttpRequest, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use quanta_artifact::{Artifact, MagnetLink};

use crate::{
    http::{
        error::QuantaHttpResponse,
        magnet::MagnetLinkListResponse,
        util::generate_error_response,
    },
    state::HttpServerState,
};
/// Name of field in [Multipart]
const FILE_MULTIPART_FORM_FIELD_NAME: &str = "file";
/// Upload InputFile into Network.
pub async fn network_file_upload_handler(
    mut payload: Multipart,
    request: HttpRequest,
    state: Data<HttpServerState>,
) -> QuantaHttpResponse {
    while let Some(Ok(mut field)) = payload.next().await {
        // check if name of field == "file"
        if field.name() == FILE_MULTIPART_FORM_FIELD_NAME {
            // we cannot create magnet link if we dont know file name. So in
            // situations if we does not get Some we are return error
            let Some(file_name) = field.content_disposition().get_filename() else {
                return generate_error_response("Filename cannot be empty");
            };
            // we cannot create magnet link if we dont know size of input file. So in
            // situations if we does not get Some we are return error
            let Some(size) = request.headers().get(actix_web::http::header::CONTENT_LENGTH) else {
                return generate_error_response("Content-Length Header Not Provided")
            };
            let size = size.to_str()?.parse::<usize>()?;
            let file_name = file_name.to_string();
            // Create magnet link which updates when we are read new artifact
            let mut magnet_link = MagnetLink::new(file_name, size);
            // Start read field with input file
            while let Ok(Some(bytes)) = field.try_next().await {
                // vec-based bytes for creating artifact
                let data = bytes.to_vec();
                let artifact = Artifact::new(data);
                let artifact_id = artifact.id;
                magnet_link.new_update_with_artifact_id(artifact_id);
                state
                    .database()
                    .insert_artifact(artifact)?;
            }
            // when read is compeleted whe should save magnet link in storage
            let magnet_string = magnet_link.to_string();
            let index = state
                .database()
                .insert_magnet_link(magnet_link)?;
            // return the StatusResponse which indicates that the file was successfully uploaded
            return Ok(HttpResponse::Ok().json(MagnetLinkListResponse {
                id: index,
                magnet: magnet_string,
            }));
        }
    }

    generate_error_response("Field 'file' does not provided in payload")
}

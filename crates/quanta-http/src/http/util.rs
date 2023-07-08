#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatusResponse<'a> {
    pub status: &'a str,
}

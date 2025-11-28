use serde::Serialize;

#[derive(Serialize)]
pub struct ImageMetadata {
    pub id: String,
    pub url: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub id: String,
    pub hash: String,
}

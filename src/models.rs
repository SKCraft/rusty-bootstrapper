use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateMeta {
    pub version: String,
    pub url: String,
}
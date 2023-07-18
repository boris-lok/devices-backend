#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResposne {
    pub status_code: u16,
    pub error_message: String,
}

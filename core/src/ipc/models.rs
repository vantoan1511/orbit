use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeutralinoConfig {
    pub nl_port: u16,
    pub nl_token: String,
    pub nl_connect_token: String,
    pub nl_extension_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: String,
    pub api_key_env: String,
    pub model: String,
    pub enabled: bool,
    pub network_required: bool,
    pub dependency_level: String,
    pub allowed_task_types: Vec<String>,
    pub max_payload_policy: serde_json::Value,
    pub sensitive_data_policy: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderHealth {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub model: String,
    pub source_kind: String,
    pub endpoint: String,
    pub enabled: bool,
    pub status: String,
    pub health: String,
    pub api_key_status: String,
    pub dependency_level: String,
    pub network_required: bool,
    pub allowed_task_types: Vec<String>,
    pub last_error: Option<String>,
    pub last_checked_at: String,
}

pub trait AIProvider {
    fn query(&self, prompt: &str) -> Result<String, String>;
    fn get_name(&self) -> &str;
    fn get_model(&self) -> &str;
}

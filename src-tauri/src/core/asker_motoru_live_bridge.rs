use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruLiveConfig {
    pub schema_version: u32,
    pub windows_root: Option<String>,
    pub linux_root: Option<String>,
    pub roots: Option<Vec<String>>,
    pub live_api: Option<LiveApiConfig>,
    pub status_files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveApiConfig {
    pub enabled: bool,
    pub api_base_url: String,
    pub ws_url: Option<String>,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStatusSnapshot {
    pub connected: bool,
    pub api_base_url: String,
    pub health: String,
    pub planning_status: Option<String>,
    pub education_status: Option<String>,
    pub alarm_status: Option<String>,
    pub last_error: Option<String>,
    pub module_count_hint: Option<u32>,
}

pub struct AskerMotoruLiveBridge;

impl AskerMotoruLiveBridge {
    pub fn load_config() -> Result<AskerMotoruLiveConfig, String> {
        let path = DependencyAnalyzer::get_config_path("asker_motoru_bridge.json")?;
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("asker_motoru_bridge.json okunamadı: {}", e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("asker_motoru_bridge.json geçersiz: {}", e))
    }

    pub fn fetch_live_status() -> LiveStatusSnapshot {
        let config = Self::load_config().unwrap_or(AskerMotoruLiveConfig {
            schema_version: 1,
            windows_root: None,
            linux_root: None,
            roots: None,
            live_api: Some(LiveApiConfig {
                enabled: false,
                api_base_url: "http://127.0.0.1:3100".to_string(),
                ws_url: None,
                timeout_ms: 5000,
                poll_interval_ms: 2000,
            }),
            status_files: None,
        });

        let api = config
            .live_api
            .unwrap_or(LiveApiConfig {
                enabled: false,
                api_base_url: "http://127.0.0.1:3100".to_string(),
                ws_url: None,
                timeout_ms: 5000,
                poll_interval_ms: 2000,
            });

        if !api.enabled {
            return LiveStatusSnapshot {
                connected: false,
                api_base_url: api.api_base_url,
                health: "disabled".to_string(),
                planning_status: None,
                education_status: None,
                alarm_status: None,
                last_error: Some("Canlı API köprüsü devre dışı. Dosya köprüsü aktif.".to_string()),
                module_count_hint: Some(314),
            };
        }

        let url = format!("{}/api/status", api.api_base_url.trim_end_matches('/'));
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_millis(api.timeout_ms))
            .build();

        match agent.get(&url).call() {
            Ok(response) => {
                let body = response.into_string().unwrap_or_default();
                let health = if body.to_ascii_lowercase().contains("healthy") {
                    "healthy"
                } else {
                    "degraded"
                };
                LiveStatusSnapshot {
                    connected: true,
                    api_base_url: api.api_base_url,
                    health: health.to_string(),
                    planning_status: Self::extract_field(&body, "planning"),
                    education_status: Self::extract_field(&body, "education"),
                    alarm_status: Self::extract_field(&body, "alarm"),
                    last_error: None,
                    module_count_hint: Some(314),
                }
            }
            Err(err) => LiveStatusSnapshot {
                connected: false,
                api_base_url: api.api_base_url,
                health: "unavailable".to_string(),
                planning_status: None,
                education_status: None,
                alarm_status: None,
                last_error: Some(format!("Canlı API erişilemedi: {err}")),
                module_count_hint: Some(314),
            },
        }
    }

    pub fn post_command(sentence: &str) -> Result<String, String> {
        let config = Self::load_config()?;
        let api = config
            .live_api
            .ok_or_else(|| "live_api yapılandırması yok.".to_string())?;
        if !api.enabled {
            return Err("Canlı API köprüsü devre dışı.".to_string());
        }
        let url = format!("{}/api/command", api.api_base_url.trim_end_matches('/'));
        let payload = serde_json::json!({ "sentence": sentence, "locale": "tr-TR" });
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_millis(api.timeout_ms))
            .build();
        let response = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_string(&payload.to_string())
            .map_err(|e| format!("Asker Motoru komut gönderimi başarısız: {e}"))?;
        response
            .into_string()
            .map_err(|e| format!("Asker Motoru yanıtı okunamadı: {e}"))
    }

    fn extract_field(body: &str, key: &str) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|value| value.get(key).map(|item| item.to_string()))
    }
}

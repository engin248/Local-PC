use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinokioConnectorConfig {
    pub control_plane_url: String,
    pub pterm_path: Option<String>,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinokioAppStatus {
    pub app_id: String,
    pub running: bool,
    pub ready: bool,
    pub ready_url: Option<String>,
    pub state: String,
    pub last_error: Option<String>,
}

pub struct PinokioConnector;

impl PinokioConnector {
    pub fn load_config() -> PinokioConnectorConfig {
        let default = PinokioConnectorConfig {
            control_plane_url: "http://127.0.0.1:42000".to_string(),
            pterm_path: None,
            timeout_ms: 5000,
        };
        let Ok(path) = DependencyAnalyzer::get_config_path("pinokio_connector.json") else {
            return default;
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or(default)
    }

    pub fn resolve_pterm_path() -> Option<String> {
        if let Ok(path) = std::env::var("PTERM_PATH") {
            if !path.trim().is_empty() {
                return Some(path);
            }
        }
        let config = Self::load_config();
        if let Some(path) = config.pterm_path {
            if !path.trim().is_empty() {
                return Some(path);
            }
        }
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        let candidates = [
            format!("{home}/.pinokio/bin/npm/bin/pterm"),
            format!("{home}/.pinokio/bin/pterm"),
        ];
        for candidate in candidates {
            if std::path::Path::new(&candidate).exists() {
                return Some(candidate);
            }
        }
        Command::new("which")
            .arg("pterm")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    pub fn health_check() -> (String, Option<String>) {
        let config = Self::load_config();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build();
        let url = format!(
            "{}/pinokio/path/pterm",
            config.control_plane_url.trim_end_matches('/')
        );
        match agent.get(&url).call() {
            Ok(_) => ("available".to_string(), None),
            Err(err) => (
                "unavailable".to_string(),
                Some(format!("Pinokio kontrol düzlemi erişilemedi: {err}")),
            ),
        }
    }

    pub fn status(app_id: &str) -> PinokioAppStatus {
        let Some(pterm) = Self::resolve_pterm_path() else {
            return PinokioAppStatus {
                app_id: app_id.to_string(),
                running: false,
                ready: false,
                ready_url: None,
                state: "offline".to_string(),
                last_error: Some("pterm bulunamadı.".to_string()),
            };
        };
        let output = Command::new(&pterm)
            .args(["status", app_id])
            .output();
        match output {
            Ok(result) if result.status.success() => {
                let body = String::from_utf8_lossy(&result.stdout).to_string();
                PinokioAppStatus {
                    app_id: app_id.to_string(),
                    running: body.contains("\"running\":true") || body.contains("running: true"),
                    ready: body.contains("\"ready\":true") || body.contains("ready: true"),
                    ready_url: Self::extract_json_string(&body, "ready_url"),
                    state: if body.contains("online") {
                        "online".to_string()
                    } else if body.contains("starting") {
                        "starting".to_string()
                    } else {
                        "offline".to_string()
                    },
                    last_error: None,
                }
            }
            Ok(result) => PinokioAppStatus {
                app_id: app_id.to_string(),
                running: false,
                ready: false,
                ready_url: None,
                state: "offline".to_string(),
                last_error: Some(String::from_utf8_lossy(&result.stderr).to_string()),
            },
            Err(err) => PinokioAppStatus {
                app_id: app_id.to_string(),
                running: false,
                ready: false,
                ready_url: None,
                state: "offline".to_string(),
                last_error: Some(format!("pterm status çalıştırılamadı: {err}")),
            },
        }
    }

    pub fn run_app(app_id: &str) -> Result<String, String> {
        let pterm = Self::resolve_pterm_path().ok_or_else(|| "pterm bulunamadı.".to_string())?;
        let output = Command::new(&pterm)
            .args(["run", app_id])
            .output()
            .map_err(|e| format!("pterm run başarısız: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn extract_json_string(body: &str, key: &str) -> Option<String> {
        let marker = format!("\"{key}\":\"");
        let start = body.find(&marker)? + marker.len();
        let rest = &body[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }
}

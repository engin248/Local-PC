use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruStatusFile {
    pub root_id: String,
    pub root_role: String,
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRootReport {
    pub id: String,
    pub role: String,
    pub configured_path: String,
    pub resolved_path: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEndpointContract {
    pub name: String,
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEndpointStatus {
    pub name: String,
    pub method: String,
    pub path: String,
    pub url: String,
    pub status: String,
    pub http_status: Option<u16>,
    pub preview: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruApiReport {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_ms: u64,
    pub endpoints: Vec<AskerMotoruEndpointStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruBridgeReport {
    pub roots_checked: Vec<String>,
    pub roots: Vec<AskerMotoruRootReport>,
    pub files: Vec<AskerMotoruStatusFile>,
    pub contract: Vec<AskerMotoruEndpointContract>,
    pub api: AskerMotoruApiReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruBridgeConfig {
    pub schema_version: u32,
    pub roots: Vec<AskerMotoruRootConfig>,
    pub status_files: Vec<String>,
    pub api: AskerMotoruApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRootConfig {
    pub id: String,
    pub path: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruApiConfig {
    pub enabled: bool,
    pub base_url: String,
    pub timeout_ms: Option<u64>,
    pub endpoints: AskerMotoruEndpointsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEndpointsConfig {
    pub health: AskerMotoruEndpointConfig,
    pub status: AskerMotoruEndpointConfig,
    pub events: AskerMotoruEndpointConfig,
    pub command: AskerMotoruEndpointConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEndpointConfig {
    pub method: String,
    pub path: String,
}

pub struct AskerMotoruBridge;

impl AskerMotoruBridge {
    pub fn scan_status_files() -> Result<AskerMotoruBridgeReport, String> {
        let config = Self::load_config()?;
        Self::scan_status_files_with_config(&config)
    }

    pub fn load_config() -> Result<AskerMotoruBridgeConfig, String> {
        let path = DependencyAnalyzer::get_config_path("asker_motoru.json")?;
        let data =
            fs::read_to_string(&path).map_err(|e| format!("asker_motoru.json okunamadı: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("asker_motoru.json geçersiz: {}", e))
    }

    pub fn scan_status_files_with_config(
        config: &AskerMotoruBridgeConfig,
    ) -> Result<AskerMotoruBridgeReport, String> {
        let mut files = Vec::new();
        let mut roots_checked = Vec::new();
        let mut roots = Vec::new();

        for root in &config.roots {
            let resolved_root = Self::resolve_path(&root.path)?;
            let resolved_text = resolved_root.display().to_string();
            roots_checked.push(resolved_text.clone());
            roots.push(AskerMotoruRootReport {
                id: root.id.clone(),
                role: root.role.clone(),
                configured_path: root.path.clone(),
                resolved_path: resolved_text,
                exists: resolved_root.exists(),
            });
            if !resolved_root.exists() {
                continue;
            }
            for name in &config.status_files {
                let path = resolved_root.join(name);
                let exists = path.exists();
                let preview = if exists {
                    fs::read_to_string(&path)
                        .map(|text| Self::preview_text(&text))
                        .unwrap_or_else(|e| format!("okunamadı: {}", e))
                } else {
                    "dosya yok".to_string()
                };
                files.push(AskerMotoruStatusFile {
                    root_id: root.id.clone(),
                    root_role: root.role.clone(),
                    name: name.clone(),
                    path: path.display().to_string(),
                    exists,
                    preview,
                });
            }
        }

        let contract = Self::endpoint_contract(&config.api.endpoints);
        let api = Self::check_api(&config.api, &contract);

        Ok(AskerMotoruBridgeReport {
            roots_checked,
            roots,
            files,
            contract,
            api,
        })
    }

    fn resolve_path(path: &str) -> Result<PathBuf, String> {
        let root = DependencyAnalyzer::get_project_root()?;
        let parent = root.parent().unwrap_or(&root);
        let resolved = path
            .replace("$PROJECT_ROOT", &root.to_string_lossy())
            .replace("$PARENT_DIR", &parent.to_string_lossy());
        Ok(PathBuf::from(resolved))
    }

    fn endpoint_contract(
        endpoints: &AskerMotoruEndpointsConfig,
    ) -> Vec<AskerMotoruEndpointContract> {
        [
            ("health", &endpoints.health),
            ("status", &endpoints.status),
            ("events", &endpoints.events),
            ("command", &endpoints.command),
        ]
        .into_iter()
        .map(|(name, endpoint)| AskerMotoruEndpointContract {
            name: name.to_string(),
            method: endpoint.method.clone(),
            path: endpoint.path.clone(),
        })
        .collect()
    }

    fn check_api(
        api: &AskerMotoruApiConfig,
        contract: &[AskerMotoruEndpointContract],
    ) -> AskerMotoruApiReport {
        let timeout_ms = api.timeout_ms.unwrap_or(1500);
        let endpoints = contract
            .iter()
            .map(|endpoint| Self::check_endpoint(api, endpoint, timeout_ms))
            .collect();

        AskerMotoruApiReport {
            enabled: api.enabled,
            base_url: api.base_url.clone(),
            timeout_ms,
            endpoints,
        }
    }

    fn check_endpoint(
        api: &AskerMotoruApiConfig,
        endpoint: &AskerMotoruEndpointContract,
        timeout_ms: u64,
    ) -> AskerMotoruEndpointStatus {
        let url = Self::join_url(&api.base_url, &endpoint.path);
        if !api.enabled {
            return AskerMotoruEndpointStatus {
                name: endpoint.name.clone(),
                method: endpoint.method.clone(),
                path: endpoint.path.clone(),
                url,
                status: "disabled".to_string(),
                http_status: None,
                preview: None,
                last_error: None,
            };
        }

        if endpoint.method.to_uppercase() != "GET" {
            return AskerMotoruEndpointStatus {
                name: endpoint.name.clone(),
                method: endpoint.method.clone(),
                path: endpoint.path.clone(),
                url,
                status: "contract_only".to_string(),
                http_status: None,
                preview: None,
                last_error: Some("Komut endpoint'i health taramasında çalıştırılmaz.".to_string()),
            };
        }

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .build();
        match agent.get(&url).call() {
            Ok(response) => {
                let http_status = response.status();
                let preview = response
                    .into_string()
                    .ok()
                    .map(|text| Self::preview_text(&text));
                AskerMotoruEndpointStatus {
                    name: endpoint.name.clone(),
                    method: endpoint.method.clone(),
                    path: endpoint.path.clone(),
                    url,
                    status: if http_status < 400 {
                        "available".to_string()
                    } else {
                        "error".to_string()
                    },
                    http_status: Some(http_status),
                    preview,
                    last_error: None,
                }
            }
            Err(ureq::Error::Status(status, response)) => AskerMotoruEndpointStatus {
                name: endpoint.name.clone(),
                method: endpoint.method.clone(),
                path: endpoint.path.clone(),
                url,
                status: "error".to_string(),
                http_status: Some(status),
                preview: response
                    .into_string()
                    .ok()
                    .map(|text| Self::preview_text(&text)),
                last_error: Some(format!("HTTP {}", status)),
            },
            Err(error) => AskerMotoruEndpointStatus {
                name: endpoint.name.clone(),
                method: endpoint.method.clone(),
                path: endpoint.path.clone(),
                url,
                status: "connection_failed".to_string(),
                http_status: None,
                preview: None,
                last_error: Some(error.to_string()),
            },
        }
    }

    fn join_url(base_url: &str, path: &str) -> String {
        format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn preview_text(text: &str) -> String {
        let mut preview = text.chars().take(400).collect::<String>();
        if text.chars().count() > 400 {
            preview.push_str("...");
            preview
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_status_files_from_configured_roots() {
        let temp_root =
            std::env::temp_dir().join(format!("asker_motoru_bridge_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(&temp_root).unwrap();
        let status_file = temp_root.join("PLANLAMA_DURUMU.json");
        fs::write(&status_file, r#"{"status":"ok"}"#).unwrap();

        let config = AskerMotoruBridgeConfig {
            schema_version: 1,
            roots: vec![AskerMotoruRootConfig {
                id: "test_root".to_string(),
                path: temp_root.to_string_lossy().into_owned(),
                role: "active".to_string(),
            }],
            status_files: vec!["PLANLAMA_DURUMU.json".to_string()],
            api: AskerMotoruApiConfig {
                enabled: false,
                base_url: "http://127.0.0.1:8090".to_string(),
                timeout_ms: Some(1500),
                endpoints: AskerMotoruEndpointsConfig {
                    health: AskerMotoruEndpointConfig {
                        method: "GET".to_string(),
                        path: "/health".to_string(),
                    },
                    status: AskerMotoruEndpointConfig {
                        method: "GET".to_string(),
                        path: "/status".to_string(),
                    },
                    events: AskerMotoruEndpointConfig {
                        method: "GET".to_string(),
                        path: "/events".to_string(),
                    },
                    command: AskerMotoruEndpointConfig {
                        method: "POST".to_string(),
                        path: "/command".to_string(),
                    },
                },
            },
        };

        let report = AskerMotoruBridge::scan_status_files_with_config(&config).unwrap();
        assert_eq!(report.files.len(), 1);
        assert!(report.files[0].exists);
        assert!(report.files[0].preview.contains("\"status\":\"ok\""));
        assert_eq!(
            report
                .contract
                .iter()
                .map(|endpoint| endpoint.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/health", "/status", "/events", "/command"]
        );

        let _ = fs::remove_dir_all(&temp_root);
    }
}

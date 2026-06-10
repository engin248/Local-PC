use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRootConfig {
    pub id: String,
    pub label: String,
    pub path: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEndpointContract {
    pub path: String,
    pub method: String,
    pub description: String,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruContract {
    pub health: AskerMotoruEndpointContract,
    pub status: AskerMotoruEndpointContract,
    pub events: AskerMotoruEndpointContract,
    pub command: AskerMotoruEndpointContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruConfig {
    pub roots: Vec<AskerMotoruRootConfig>,
    pub status_files: Vec<String>,
    pub contract: AskerMotoruContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruStatusFile {
    pub root_id: String,
    pub path: String,
    pub exists: bool,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRootStatus {
    pub id: String,
    pub label: String,
    pub path: String,
    pub exists: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruHealthContract {
    pub path: String,
    pub method: String,
    pub status: String,
    pub roots_total: usize,
    pub roots_available: usize,
    pub files_total: usize,
    pub files_available: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRuntimeStatus {
    pub path: String,
    pub method: String,
    pub status_files_total: usize,
    pub status_files_available: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEventItem {
    pub source: String,
    pub path: String,
    pub exists: bool,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruEventsContract {
    pub path: String,
    pub method: String,
    pub sources: Vec<AskerMotoruEventItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruCommandContract {
    pub path: String,
    pub method: String,
    pub allowed_commands: Vec<String>,
    pub available: bool,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruBridgeReport {
    pub contract: AskerMotoruContract,
    pub health: AskerMotoruHealthContract,
    pub status: AskerMotoruRuntimeStatus,
    pub events: AskerMotoruEventsContract,
    pub command: AskerMotoruCommandContract,
    pub roots: Vec<AskerMotoruRootStatus>,
    pub roots_checked: Vec<String>,
    pub files: Vec<AskerMotoruStatusFile>,
}

pub struct AskerMotoruBridge;

impl AskerMotoruBridge {
    pub fn scan_status_files() -> Result<AskerMotoruBridgeReport, String> {
        let path = DependencyAnalyzer::get_config_path("asker_motoru.json")?;
        let data =
            fs::read_to_string(&path).map_err(|e| format!("asker_motoru.json okunamadı: {}", e))?;
        let config: AskerMotoruConfig = serde_json::from_str(&data)
            .map_err(|e| format!("asker_motoru.json geçersiz: {}", e))?;
        Self::scan_status_files_with_config(&config)
    }

    fn scan_status_files_with_config(
        config: &AskerMotoruConfig,
    ) -> Result<AskerMotoruBridgeReport, String> {
        if config.roots.is_empty() {
            return Err("asker_motoru.json roots listesi boş.".to_string());
        }
        if config.status_files.is_empty() {
            return Err("asker_motoru.json status_files listesi boş.".to_string());
        }

        let mut files = Vec::new();
        let mut roots = Vec::new();
        let mut roots_checked = Vec::new();
        let mut last_error = None;

        for root_config in &config.roots {
            let root = DependencyAnalyzer::resolve_configured_path(&root_config.path)?;
            let root_path = normalize_path(root);
            let root_exists = root_path.exists();
            roots_checked.push(root_path.display().to_string());
            roots.push(AskerMotoruRootStatus {
                id: root_config.id.clone(),
                label: root_config.label.clone(),
                path: root_path.display().to_string(),
                exists: root_exists,
                required: root_config.required,
            });
            if !root_exists {
                if root_config.required {
                    last_error = Some(format!(
                        "Zorunlu Asker Motoru kökü bulunamadı: {}",
                        root_path.display()
                    ));
                }
                continue;
            }
            for name in &config.status_files {
                let path = root_path.join(name);
                let exists = path.exists();
                let preview = read_preview(&path, exists);
                files.push(AskerMotoruStatusFile {
                    root_id: root_config.id.clone(),
                    path: path.display().to_string(),
                    exists,
                    preview,
                });
            }
        }

        let roots_available = roots.iter().filter(|root| root.exists).count();
        let files_available = files.iter().filter(|file| file.exists).count();
        let files_total = roots_available * config.status_files.len();
        let required_missing = roots.iter().any(|root| root.required && !root.exists);
        let health_status = if required_missing {
            "error"
        } else if roots_available > 0 && files_available > 0 {
            "available"
        } else {
            "degraded"
        }
        .to_string();
        let event_sources = files
            .iter()
            .filter(|file| {
                file.path.contains("ALARM") || file.path.contains("SON_PLANLAMA_OPERASYONU")
            })
            .map(|file| AskerMotoruEventItem {
                source: file.root_id.clone(),
                path: file.path.clone(),
                exists: file.exists,
                preview: file.preview.clone(),
            })
            .collect();

        Ok(AskerMotoruBridgeReport {
            contract: config.contract.clone(),
            health: AskerMotoruHealthContract {
                path: config.contract.health.path.clone(),
                method: config.contract.health.method.clone(),
                status: health_status,
                roots_total: roots.len(),
                roots_available,
                files_total,
                files_available,
                last_error,
            },
            status: AskerMotoruRuntimeStatus {
                path: config.contract.status.path.clone(),
                method: config.contract.status.method.clone(),
                status_files_total: files_total,
                status_files_available: files_available,
            },
            events: AskerMotoruEventsContract {
                path: config.contract.events.path.clone(),
                method: config.contract.events.method.clone(),
                sources: event_sources,
            },
            command: AskerMotoruCommandContract {
                path: config.contract.command.path.clone(),
                method: config.contract.command.method.clone(),
                allowed_commands: config.contract.command.allowed_commands.clone(),
                available: roots_available > 0,
                requires_approval: true,
            },
            roots,
            roots_checked,
            files,
        })
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if path.components().count() == 0 {
        return path;
    }
    path
}

fn read_preview(path: &PathBuf, exists: bool) -> String {
    if !exists {
        return "dosya yok".to_string();
    }
    fs::read_to_string(path)
        .map(|text| {
            let preview: String = text.chars().take(400).collect();
            if text.chars().count() > 400 {
                format!("{}...", preview)
            } else {
                text
            }
        })
        .unwrap_or_else(|e| format!("okunamadı: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(path: &str, method: &str) -> AskerMotoruEndpointContract {
        AskerMotoruEndpointContract {
            path: path.to_string(),
            method: method.to_string(),
            description: format!("{method} {path}"),
            allowed_commands: Vec::new(),
        }
    }

    #[test]
    fn configured_roots_and_contract_drive_bridge_report() {
        let root =
            std::env::temp_dir().join(format!("asker_motoru_bridge_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("PLANLAMA_DURUMU.json"), r#"{"durum":"hazir"}"#).unwrap();
        fs::write(root.join("SISTEM_ALARM_DURUMU.json"), r#"{"alarm":false}"#).unwrap();

        let mut command = endpoint("/command", "POST");
        command.allowed_commands = vec!["refresh_status".to_string()];
        let config = AskerMotoruConfig {
            roots: vec![AskerMotoruRootConfig {
                id: "test_root".to_string(),
                label: "Test Root".to_string(),
                path: root.to_string_lossy().into_owned(),
                required: true,
            }],
            status_files: vec![
                "PLANLAMA_DURUMU.json".to_string(),
                "SISTEM_ALARM_DURUMU.json".to_string(),
            ],
            contract: AskerMotoruContract {
                health: endpoint("/health", "GET"),
                status: endpoint("/status", "GET"),
                events: endpoint("/events", "GET"),
                command,
            },
        };

        let report = AskerMotoruBridge::scan_status_files_with_config(&config).unwrap();
        assert_eq!(report.health.path, "/health");
        assert_eq!(report.status.path, "/status");
        assert_eq!(report.events.path, "/events");
        assert_eq!(report.command.path, "/command");
        assert_eq!(report.health.status, "available");
        assert_eq!(report.health.roots_available, 1);
        assert_eq!(report.health.files_available, 2);
        assert_eq!(report.events.sources.len(), 1);

        let _ = fs::remove_dir_all(root);
    }
}

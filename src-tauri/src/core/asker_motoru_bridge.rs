use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruStatusFile {
    pub path: String,
    pub exists: bool,
    pub source_kind: String,
    pub health: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruRootStatus {
    pub kind: String,
    pub source_kind: String,
    pub source_path: Option<String>,
    pub health: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruBridgeReport {
    pub roots_checked: Vec<String>,
    pub root_sources: Vec<AskerMotoruRootStatus>,
    pub files: Vec<AskerMotoruStatusFile>,
}

pub struct AskerMotoruBridge;

impl AskerMotoruBridge {
    pub fn scan_status_files() -> AskerMotoruBridgeReport {
        let roots = Self::configured_roots();
        let candidates = Self::status_file_names();

        let mut files = Vec::new();
        let mut roots_checked = Vec::new();
        let mut root_sources = Vec::new();

        for (kind, root) in roots {
            roots_checked.push(root.display().to_string());
            root_sources.push(Self::root_status(&kind, &root));
            if !root.exists() {
                continue;
            }
            for name in &candidates {
                let path = root.join(name);
                let exists = path.exists();
                let preview = if exists {
                    fs::read_to_string(&path)
                        .map(|text| {
                            if text.len() > 400 {
                                format!("{}...", &text[..400])
                            } else {
                                text
                            }
                        })
                        .unwrap_or_else(|e| format!("okunamadı: {}", e))
                } else {
                    "dosya yok".to_string()
                };
                files.push(AskerMotoruStatusFile {
                    path: path.display().to_string(),
                    exists,
                    source_kind: if exists { "json" } else { "unavailable" }.to_string(),
                    health: if exists { "available" } else { "unavailable" }.to_string(),
                    preview,
                });
            }
        }

        if root_sources.is_empty() {
            roots_checked.push("windows:unavailable".to_string());
            roots_checked.push("linux:unavailable".to_string());
            root_sources.push(AskerMotoruRootStatus {
                kind: "windows".to_string(),
                source_kind: "unavailable".to_string(),
                source_path: None,
                health: "unavailable".to_string(),
                error: Some("ASKER_MOTORU_WINDOWS_ROOT veya config/asker_motoru_bridge.json tanımlı değil.".to_string()),
            });
            root_sources.push(AskerMotoruRootStatus {
                kind: "linux".to_string(),
                source_kind: "unavailable".to_string(),
                source_path: None,
                health: "unavailable".to_string(),
                error: Some("ASKER_MOTORU_LINUX_ROOT veya config/asker_motoru_bridge.json tanımlı değil.".to_string()),
            });
        }

        AskerMotoruBridgeReport {
            roots_checked,
            root_sources,
            files,
        }
    }

    pub fn read_alarm_status_file() -> Option<(String, String)> {
        for (_, root) in Self::configured_roots() {
            let path = root.join("SISTEM_ALARM_DURUMU.json");
            if path.exists() {
                let content = fs::read_to_string(&path).ok()?;
                return Some((path.display().to_string(), content));
            }
        }
        None
    }

    fn configured_roots() -> Vec<(String, PathBuf)> {
        let mut roots = Vec::new();
        if let Ok(path) = std::env::var("ASKER_MOTORU_WINDOWS_ROOT") {
            if !path.trim().is_empty() {
                roots.push(("windows".to_string(), PathBuf::from(path)));
            }
        }
        if let Ok(path) = std::env::var("ASKER_MOTORU_LINUX_ROOT") {
            if !path.trim().is_empty() {
                roots.push(("linux".to_string(), PathBuf::from(path)));
            }
        }
        if let Ok(path) = std::env::var("ASKER_MOTORU_ROOT") {
            if !path.trim().is_empty() {
                roots.push(("generic".to_string(), PathBuf::from(path)));
            }
        }
        roots.extend(Self::config_roots());
        roots
    }

    fn config_roots() -> Vec<(String, PathBuf)> {
        let Ok(config_path) = crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path("asker_motoru_bridge.json") else {
            return Vec::new();
        };
        let Ok(data) = fs::read_to_string(config_path) else {
            return Vec::new();
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else {
            return Vec::new();
        };
        let mut roots = Vec::new();
        for key in ["windows_root", "linux_root", "root"] {
            if let Some(path) = value.get(key).and_then(|item| item.as_str()) {
                if !path.trim().is_empty() {
                    roots.push((key.trim_end_matches("_root").to_string(), PathBuf::from(path)));
                }
            }
        }
        if let Some(items) = value.get("roots").and_then(|item| item.as_array()) {
            for item in items {
                if let Some(path) = item.as_str() {
                    roots.push(("config".to_string(), PathBuf::from(path)));
                }
            }
        }
        roots
    }

    fn status_file_names() -> Vec<String> {
        if let Ok(raw) = std::env::var("ASKER_MOTORU_STATUS_FILES") {
            let names: Vec<String> = raw
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToString::to_string)
                .collect();
            if !names.is_empty() {
                return names;
            }
        }
        vec![
            "PLANLAMA_DURUMU.json".to_string(),
            "SISTEM_ALARM_DURUMU.json".to_string(),
            "EGITIM_DURUMU.json".to_string(),
            "SON_PLANLAMA_OPERASYONU.json".to_string(),
        ]
    }

    fn root_status(kind: &str, root: &PathBuf) -> AskerMotoruRootStatus {
        let exists = root.exists();
        AskerMotoruRootStatus {
            kind: kind.to_string(),
            source_kind: if exists { "json" } else { "unavailable" }.to_string(),
            source_path: Some(root.display().to_string()),
            health: if exists { "available" } else { "unavailable" }.to_string(),
            error: if exists {
                None
            } else {
                Some(format!("Asker Motoru {} path bulunamadı: {}", kind, root.display()))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AskerMotoruBridge;

    #[test]
    fn missing_bridge_config_reports_windows_and_linux_unavailable() {
        std::env::remove_var("ASKER_MOTORU_WINDOWS_ROOT");
        std::env::remove_var("ASKER_MOTORU_LINUX_ROOT");
        std::env::remove_var("ASKER_MOTORU_ROOT");

        let report = AskerMotoruBridge::scan_status_files();

        assert!(report
            .root_sources
            .iter()
            .any(|root| root.kind == "windows" && root.source_kind == "unavailable"));
        assert!(report
            .root_sources
            .iter()
            .any(|root| root.kind == "linux" && root.source_kind == "unavailable"));
    }
}

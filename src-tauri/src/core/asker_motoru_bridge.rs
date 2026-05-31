use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruStatusFile {
    pub path: String,
    pub exists: bool,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruBridgeReport {
    pub roots_checked: Vec<String>,
    pub files: Vec<AskerMotoruStatusFile>,
}

pub struct AskerMotoruBridge;

impl AskerMotoruBridge {
    pub fn scan_status_files() -> AskerMotoruBridgeReport {
        let roots = vec![
            PathBuf::from(r"C:\Users\Esisya\Desktop\asker motoru"),
            PathBuf::from(r"C:\Users\Esisya\Desktop\ASKER_MOTORU_KOK_KLASORU"),
        ];
        let candidates = [
            "PLANLAMA_DURUMU.json",
            "SISTEM_ALARM_DURUMU.json",
            "EGITIM_DURUMU.json",
            "SON_PLANLAMA_OPERASYONU.json",
        ];

        let mut files = Vec::new();
        let mut roots_checked = Vec::new();

        for root in roots {
            roots_checked.push(root.display().to_string());
            if !root.exists() {
                continue;
            }
            for name in candidates {
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
                    preview,
                });
            }
        }

        AskerMotoruBridgeReport {
            roots_checked,
            files,
        }
    }
}

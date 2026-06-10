use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::asker_motoru_modules::{CapabilityBundle, ASKER_MOTORU_MODULES};

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
    pub module_summary: AskerMotoruModuleSummary,
    pub modules: Vec<AskerMotoruModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruModuleSummary {
    pub total_modules: usize,
    pub total_specialty_capabilities: usize,
    pub capability_bundles: Vec<AskerMotoruCapabilityBundleSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruCapabilityBundleSummary {
    pub capability_bundle: String,
    pub label: String,
    pub module_count: usize,
    pub specialty_capability_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerMotoruModule {
    pub module_name: String,
    pub capability_bundle: String,
    pub specialty_capabilities: Vec<String>,
}

pub struct AskerMotoruBridge;

impl AskerMotoruBridge {
    pub fn scan_status_files() -> AskerMotoruBridgeReport {
        let modules = Self::module_catalog();
        let module_summary = Self::summarize_modules();
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
            module_summary,
            modules,
        }
    }

    pub fn module_catalog() -> Vec<AskerMotoruModule> {
        ASKER_MOTORU_MODULES
            .iter()
            .map(|entry| {
                let specialty_capabilities = entry
                    .capability_bundle
                    .specialty_capabilities()
                    .iter()
                    .map(|capability| (*capability).to_string())
                    .collect();

                AskerMotoruModule {
                    module_name: entry.module_name.to_string(),
                    capability_bundle: entry.capability_bundle.id().to_string(),
                    specialty_capabilities,
                }
            })
            .collect()
    }

    fn summarize_modules() -> AskerMotoruModuleSummary {
        let mut counts: BTreeMap<CapabilityBundle, usize> = BTreeMap::new();
        let mut total_specialty_capabilities = 0;

        for entry in ASKER_MOTORU_MODULES {
            *counts.entry(entry.capability_bundle).or_insert(0) += 1;
            total_specialty_capabilities += entry.capability_bundle.specialty_capabilities().len();
        }

        let capability_bundles = counts
            .into_iter()
            .map(
                |(bundle, module_count)| AskerMotoruCapabilityBundleSummary {
                    capability_bundle: bundle.id().to_string(),
                    label: bundle.label().to_string(),
                    module_count,
                    specialty_capability_count: module_count
                        * bundle.specialty_capabilities().len(),
                },
            )
            .collect();

        AskerMotoruModuleSummary {
            total_modules: ASKER_MOTORU_MODULES.len(),
            total_specialty_capabilities,
            capability_bundles,
        }
    }
}

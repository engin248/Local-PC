use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLibraryConfig {
    pub schema_version: u32,
    pub sqlite_path: Option<String>,
    pub env_var: Option<String>,
    pub windows_default: Option<String>,
    pub relative_candidates: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub read_only_default: Option<bool>,
}

pub struct SkillLibrary;

impl SkillLibrary {
    pub fn load_config() -> SkillLibraryConfig {
        let default = SkillLibraryConfig {
            schema_version: 1,
            sqlite_path: None,
            env_var: Some("SKILL_LIBRARY_DB_PATH".to_string()),
            windows_default: Some(
                "C:\\Users\\Esisya\\Desktop\\Lokal Kütüphane\\database\\skill_library.sqlite"
                    .to_string(),
            ),
            relative_candidates: Some(vec![
                "../Lokal Kütüphane/database/skill_library.sqlite".to_string(),
                "storage/skill_library.sqlite".to_string(),
            ]),
            enabled: Some(false),
            read_only_default: Some(true),
        };
        let Ok(path) = DependencyAnalyzer::get_config_path("skill_library.json") else {
            return default;
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(default)
    }

    pub fn resolve_db_path() -> Option<PathBuf> {
        let config = Self::load_config();
        if let Some(key) = &config.env_var {
            if let Ok(path) = std::env::var(key) {
                if !path.trim().is_empty() {
                    let candidate = PathBuf::from(path);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
        if let Some(path) = &config.sqlite_path {
            if !path.trim().is_empty() {
                let candidate = PathBuf::from(path);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
        if let Some(path) = &config.windows_default {
            let candidate = PathBuf::from(path);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if let Ok(root) = DependencyAnalyzer::get_project_root() {
            if let Some(items) = &config.relative_candidates {
                for item in items {
                    let candidate = root.join(item);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }
}

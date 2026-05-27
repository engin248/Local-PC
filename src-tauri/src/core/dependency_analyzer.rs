use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConnectorConfig {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub connector_type: String,
    pub path: Option<String>,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub dependency_level: Option<String>,
    pub live_system: Option<bool>,
    pub network_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub model: Option<String>,
    pub enabled: Option<bool>,
    pub dependency_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAssessment {
    pub id: String,
    pub task_id: String,
    pub target_id: String,
    pub target_type: String,
    pub dependency_level: String,
    pub status: String,
    pub reason: String,
    pub network_required: bool,
    pub api_key_required: bool,
    pub live_system: bool,
    pub permissions: Vec<String>,
    pub approval_required: bool,
}

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn get_project_root() -> Result<std::path::PathBuf, String> {
        if let Ok(current_dir) = std::env::current_dir() {
            let mut check_dir = current_dir;
            loop {
                if check_dir.join("package.json").exists() || check_dir.join("src-tauri").exists() {
                    return Ok(check_dir);
                }
                if let Some(parent) = check_dir.parent() {
                    check_dir = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }
        Err("Proje kök dizini bulunamadı!".to_string())
    }

    // Dynamic config path helper that walks directories to prevent hardcoded issues
    pub fn get_config_path(filename: &str) -> Result<String, String> {
        let root = Self::get_project_root()?;
        let candidate = root.join("config").join(filename);
        if candidate.exists() {
            Ok(candidate.to_string_lossy().into_owned())
        } else {
            Err(format!("Yapılandırma dosyası ({}) bulunamadı. Lütfen config dizininin mevcut olduğunu kontrol edin.", filename))
        }
    }

    // Dynamic database persistence helper
    fn save_assessment_to_db(assessment: &DependencyAssessment) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO dependency_assessments (
                id, task_id, target_id, target_type, dependency_level, 
                status, reason, network_required, api_key_required, 
                live_system, approval_required
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                assessment.id,
                assessment.task_id,
                assessment.target_id,
                assessment.target_type,
                assessment.dependency_level,
                assessment.status,
                assessment.reason,
                if assessment.network_required { 1 } else { 0 },
                if assessment.api_key_required { 1 } else { 0 },
                if assessment.live_system { 1 } else { 0 },
                if assessment.approval_required { 1 } else { 0 }
            ],
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    // 1. Analyze System Connector Dependency
    pub fn analyze_system_connector_dependency(
        task_id: &str,
        connector_id: &str,
        config_path: &str,
    ) -> Result<DependencyAssessment, String> {
        let data = fs::read_to_string(config_path)
            .map_err(|e| format!("system_connectors.json okunamadı: {}", e))?;

        let connectors: Vec<SystemConnectorConfig> = serde_json::from_str(&data)
            .map_err(|e| format!("system_connectors.json geçersiz JSON: {}", e))?;

        let connector = connectors
            .iter()
            .find(|c| c.id == connector_id || c.connector_type == connector_id)
            .ok_or_else(|| format!("Connector bulunamadı: {}", connector_id))?;

        let enabled = connector.enabled.unwrap_or(false);
        let permissions = connector.permissions.clone().unwrap_or_default();
        let network_required = connector.network_required.unwrap_or(
            matches!(connector.connector_type.as_str(), "api" | "live_api")
        );
        let api_key_required = connector.api_key_env.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let live_system = connector.live_system.unwrap_or(false);

        if !enabled {
            let assessment = DependencyAssessment {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                target_id: connector.id.clone(),
                target_type: connector.connector_type.clone(),
                dependency_level: "critical".to_string(),
                status: "disabled".to_string(),
                reason: "Connector disabled durumda.".to_string(),
                network_required,
                api_key_required,
                live_system,
                permissions: permissions.clone(),
                approval_required: true,
            };
            
            Self::save_assessment_to_db(&assessment)?;
            return Ok(assessment);
        }

        let inferred_level = Self::infer_level(
            &connector.connector_type,
            connector.dependency_level.as_deref(),
            live_system,
        );

        let mut status = "available".to_string();
        let mut reason = "Bağımlılık analizi tamamlandı.".to_string();

        if matches!(connector.connector_type.as_str(), "file" | "folder" | "sqlite") {
            match &connector.path {
                Some(p) => {
                    let resolved_path = if p.starts_with("$PROJECT_ROOT") {
                        let root = Self::get_project_root()?;
                        let clean_suffix = p.trim_start_matches("$PROJECT_ROOT")
                            .trim_start_matches('/')
                            .trim_start_matches('\\');
                        root.join(clean_suffix).to_string_lossy().into_owned()
                    } else if p.starts_with("$PARENT_DIR") {
                        let root = Self::get_project_root()?;
                        let parent = root.parent().unwrap_or(&root);
                        let clean_suffix = p.trim_start_matches("$PARENT_DIR")
                            .trim_start_matches('/')
                            .trim_start_matches('\\');
                        parent.join(clean_suffix).to_string_lossy().into_owned()
                    } else if p.starts_with('.') {
                        let root = Self::get_project_root()?;
                        root.join(p).to_string_lossy().into_owned()
                    } else {
                        p.clone()
                    };

                    let resolved_path = resolved_path.replace("\\", "/");

                    if Path::new(&resolved_path).exists() {
                        reason = format!("Lokal path erişilebilir: {}", resolved_path);
                    } else {
                        status = "unavailable".to_string();
                        reason = format!("Lokal path bulunamadı veya erişilemedi: {}", resolved_path);
                    }
                }
                None => {
                    status = "unavailable".to_string();
                    reason = "Connector path alanı eksik.".to_string();
                }
            }
        }

        let mut dependency_level = inferred_level;
        if status == "unavailable" {
            dependency_level = "critical".to_string();
        }

        let approval_required =
            live_system ||
            dependency_level == "critical" ||
            permissions.iter().any(|p| {
                p.contains("write") || p.contains("delete") || p.contains("terminal")
            });

        let assessment = DependencyAssessment {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            target_id: connector.id.clone(),
            target_type: connector.connector_type.clone(),
            dependency_level: dependency_level.clone(),
            status,
            reason: reason.clone(),
            network_required,
            api_key_required,
            live_system,
            permissions: permissions.clone(),
            approval_required,
        };

        // Save to DB
        Self::save_assessment_to_db(&assessment)?;

        // Audit Logger with detailed metadata
        let metadata = serde_json::json!({
            "connector_id": connector.id,
            "connector_type": connector.connector_type,
            "dependency_level": dependency_level,
            "enabled": enabled,
            "network_required": network_required,
            "api_key_required": api_key_required,
            "permissions": permissions,
            "reason": reason
        });

        crate::core::audit_logger::AuditLogger::log_event(
            task_id,
            "info",
            &format!("Dinamik bağımlılık analizi yapıldı. Seviye: {}, Tip: {}", dependency_level, connector.connector_type),
            Some("Dependency Analyzer"),
            Some("dependency_checked"),
            Some(&metadata.to_string())
        )?;

        Ok(assessment)
    }

    // 2. Analyze AI Provider Dependency
    pub fn analyze_ai_provider_dependency(
        task_id: &str,
        provider_id: &str,
    ) -> Result<DependencyAssessment, String> {
        let config_path = Self::get_config_path("ai_providers.json")?;
        let data = fs::read_to_string(&config_path)
            .map_err(|e| format!("ai_providers.json okunamadı: {}", e))?;

        let providers: Vec<AIProviderConfig> = serde_json::from_str(&data)
            .map_err(|e| format!("ai_providers.json geçersiz JSON: {}", e))?;

        let provider = providers
            .iter()
            .find(|p| p.id == provider_id || p.provider_type == provider_id)
            .ok_or_else(|| format!("AI Provider bulunamadı: {}", provider_id))?;

        let enabled = provider.enabled.unwrap_or(false);
        let dependency_level = provider.dependency_level.clone().unwrap_or_else(|| "high".to_string());
        
        let assessment = DependencyAssessment {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            target_id: provider.id.clone(),
            target_type: "ai_provider".to_string(),
            dependency_level: if enabled { dependency_level } else { "critical".to_string() },
            status: if enabled { "available".to_string() } else { "disabled".to_string() },
            reason: format!("AI Provider analizi tamamlandı. Model: {:?}", provider.model),
            network_required: true,
            api_key_required: provider.api_key_env.as_ref().map(|s| !s.is_empty()).unwrap_or(false),
            live_system: false,
            permissions: vec!["execute".to_string()],
            approval_required: !enabled || provider.model.is_none(),
        };

        Self::save_assessment_to_db(&assessment)?;
        Ok(assessment)
    }

    // 3. Analyze Technology Dependency
    pub fn analyze_technology_dependency(
        task_id: &str,
        technology_name: &str,
    ) -> Result<DependencyAssessment, String> {
        let (dep_level, status, reason) = match technology_name.to_lowercase().as_str() {
            "rust" | "tauri" => ("low", "available", "Lokal güvenli derleme dili ve runtime."),
            "svelte" | "javascript" | "typescript" => ("low", "available", "Frontend UI kütüphanesi ve betik dili."),
            "sqlite" => ("low", "available", "Lokal dosya tabanlı veritabanı."),
            "postgresql" | "supabase" => ("high", "available", "Harici ağ veya veritabanı sunucusu bağımlılığı."),
            _ => ("critical", "available", "Bilinmeyen teknoloji bağımlılığı.")
        };

        let assessment = DependencyAssessment {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            target_id: technology_name.to_string(),
            target_type: "technology".to_string(),
            dependency_level: dep_level.to_string(),
            status: status.to_string(),
            reason: reason.to_string(),
            network_required: matches!(technology_name.to_lowercase().as_str(), "postgresql" | "supabase"),
            api_key_required: false,
            live_system: false,
            permissions: vec!["read".to_string(), "write".to_string()],
            approval_required: dep_level == "critical",
        };

        Self::save_assessment_to_db(&assessment)?;
        Ok(assessment)
    }

    // Backward-compatible bridge to prevent compiler issues elsewhere
    pub fn analyze_dependency(task_id: &str, connector_id: &str) -> Result<String, String> {
        let config_path = Self::get_config_path("system_connectors.json")?;
        
        // Check if connector_id is in system connectors
        if let Ok(assessment) = Self::analyze_system_connector_dependency(task_id, connector_id, &config_path) {
            return Ok(assessment.dependency_level);
        }

        // If not system connector, check AI providers
        if let Ok(assessment) = Self::analyze_ai_provider_dependency(task_id, connector_id) {
            return Ok(assessment.dependency_level);
        }

        // Fallback for safety
        Ok("critical".to_string())
    }

    fn infer_level(connector_type: &str, configured: Option<&str>, live_system: bool) -> String {
        if live_system {
            return "critical".to_string();
        }

        if let Some(level) = configured {
            if matches!(level, "none" | "low" | "medium" | "high" | "critical") {
                return level.to_string();
            }
            return "critical".to_string();
        }

        match connector_type {
            "file" | "folder" | "sqlite" => "low",
            "local_api" => "medium",
            "api" => "high",
            "live_api" | "terminal" => "critical",
            _ => "critical",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_level() {
        assert_eq!(DependencyAnalyzer::infer_level("mock", None, false), "critical");
        assert_eq!(DependencyAnalyzer::infer_level("folder", None, false), "low");
        assert_eq!(DependencyAnalyzer::infer_level("sqlite", None, false), "low");
        assert_eq!(DependencyAnalyzer::infer_level("local_api", None, false), "medium");
        assert_eq!(DependencyAnalyzer::infer_level("api", None, false), "high");
        assert_eq!(DependencyAnalyzer::infer_level("live_api", None, false), "critical");
        assert_eq!(DependencyAnalyzer::infer_level("terminal", None, false), "critical");
        assert_eq!(DependencyAnalyzer::infer_level("api", None, true), "critical");
        assert_eq!(DependencyAnalyzer::infer_level("api", Some("medium"), false), "medium");
        assert_eq!(DependencyAnalyzer::infer_level("api", Some("invalid_val"), false), "critical");
    }

    #[test]
    fn test_analyze_technology_dependency() {
        let db = Database::new();
        let _ = db.run_migrations();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES ('test_task', 'Test Task', 'Test Request', 'pending', 'planning_incomplete', 'not_started', 'low', 'pending_approval')",
            []
        );
        let res = DependencyAnalyzer::analyze_technology_dependency("test_task", "rust").unwrap();
        assert_eq!(res.dependency_level, "low");
        assert_eq!(res.status, "available");
        assert!(!res.approval_required);

        let res_sub = DependencyAnalyzer::analyze_technology_dependency("test_task", "supabase").unwrap();
        assert_eq!(res_sub.dependency_level, "high");
        assert_eq!(res_sub.status, "available");
        assert!(res_sub.network_required);
    }

    #[test]
    fn test_system_connector_scenarios() {
        let db = Database::new();
        let _ = db.run_migrations();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES ('test_task', 'Test Task', 'Test Request', 'pending', 'planning_incomplete', 'not_started', 'low', 'pending_approval')",
            []
        );
        // Create a temporary json file to test the 10 scenarios
        let temp_dir = std::env::temp_dir();
        let config_file_path = temp_dir.join("test_system_connectors.json");
        let config_path_str = config_file_path.to_string_lossy().to_string();

        let exists_folder = temp_dir.join("test_exists_folder");
        let _ = std::fs::create_dir_all(&exists_folder);
        let exists_folder_str = exists_folder.to_string_lossy().to_string().replace("\\", "/");

        let exists_sqlite_file = temp_dir.join("test_exists.db");
        let _ = std::fs::write(&exists_sqlite_file, b"");
        let exists_sqlite_str = exists_sqlite_file.to_string_lossy().to_string().replace("\\", "/");

        let json_content = format!(
            r#"[
                {{
                    "id": "test_mock",
                    "name": "Test Mock",
                    "type": "mock",
                    "enabled": true
                }},
                {{
                    "id": "test_folder_exists",
                    "name": "Test Folder Exists",
                    "type": "folder",
                    "path": "{}",
                    "enabled": true
                }},
                {{
                    "id": "test_folder_missing",
                    "name": "Test Folder Missing",
                    "type": "folder",
                    "path": "C:/non_existent_folder_xyz_123",
                    "enabled": true
                }},
                {{
                    "id": "test_sqlite_exists",
                    "name": "Test Sqlite Exists",
                    "type": "sqlite",
                    "path": "{}",
                    "enabled": true
                }},
                {{
                    "id": "test_api",
                    "name": "Test API",
                    "type": "api",
                    "enabled": true
                }},
                {{
                    "id": "test_live_api",
                    "name": "Test Live API",
                    "type": "live_api",
                    "live_system": true,
                    "enabled": true
                }},
                {{
                    "id": "test_disabled",
                    "name": "Test Disabled",
                    "type": "api",
                    "enabled": false
                }},
                {{
                    "id": "test_missing_level",
                    "name": "Test Missing Level",
                    "type": "unknown_type",
                    "enabled": true
                }},
                {{
                    "id": "test_write_approval",
                    "name": "Test Write Approval",
                    "type": "api",
                    "permissions": ["write_with_approval"],
                    "enabled": true
                }}
            ]"#,
            exists_folder_str, exists_sqlite_str
        );

        std::fs::write(&config_file_path, json_content).unwrap();

        // 1. mock connector is only accepted in this fixture and is treated as critical.
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_mock", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "critical");
        assert_eq!(res.status, "available");

        // 2. folder connector path var -> low
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_folder_exists", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "low");
        assert_eq!(res.status, "available");

        // 3. folder connector path yok -> critical/unavailable
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_folder_missing", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "critical");
        assert_eq!(res.status, "unavailable");

        // 4. sqlite path var -> low
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_sqlite_exists", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "low");
        assert_eq!(res.status, "available");

        // 5. api connector -> high
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_api", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "high");
        assert_eq!(res.status, "available");

        // 6. live_api connector -> critical
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_live_api", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "critical");
        assert_eq!(res.status, "available");

        // 7. enabled=false connector -> critical/disabled
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_disabled", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "critical");
        assert_eq!(res.status, "disabled");

        // 8. dependency_level eksik -> critical
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_missing_level", &config_path_str).unwrap();
        assert_eq!(res.dependency_level, "critical");

        // 9. bilinmeyen connector -> error
        let err = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_unknown_connector", &config_path_str);
        assert!(err.is_err());

        // 10. write_with_approval permission -> approval_required=true
        let res = DependencyAnalyzer::analyze_system_connector_dependency("test_task", "test_write_approval", &config_path_str).unwrap();
        assert!(res.approval_required);

        // Clean up
        let _ = std::fs::remove_file(config_file_path);
        let _ = std::fs::remove_dir_all(exists_folder);
        let _ = std::fs::remove_file(exists_sqlite_file);
    }
}

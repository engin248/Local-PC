use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use crate::system_connectors::connector_base::{SystemConnectorConfig, SystemConnectorHealth};
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SystemConnectorManager;

impl SystemConnectorManager {
    pub fn load_configs() -> Result<Vec<SystemConnectorConfig>, String> {
        let path = DependencyAnalyzer::get_config_path("system_connectors.json")?;
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("system_connectors.json okunamadı: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("system_connectors.json geçersiz: {}", e))
    }

    pub fn health_check_all(write_audit: bool) -> Result<Vec<SystemConnectorHealth>, String> {
        let configs = Self::load_configs()?;
        let mut results = Vec::new();

        for config in configs {
            let result = Self::health_check(&config);
            if write_audit {
                Self::audit_connector_health(&result)?;
            }
            results.push(result);
        }

        Ok(results)
    }

    fn health_check(config: &SystemConnectorConfig) -> SystemConnectorHealth {
        let target = config.path.clone().or_else(|| config.base_url.clone());
        let mut status = if config.enabled {
            "available"
        } else {
            "disabled"
        }
        .to_string();
        let mut last_error = None;

        if config.enabled {
            match config.connector_type.as_str() {
                "folder" | "file" => match config.path.as_deref().and_then(Self::resolve_path) {
                    Some(path) if path.exists() => {
                        status = "available".to_string();
                    }
                    Some(path) => {
                        status = "error".to_string();
                        last_error = Some(format!("Path bulunamadı: {}", path.display()));
                    }
                    None => {
                        status = "error".to_string();
                        last_error = Some("Path çözümlenemedi.".to_string());
                    }
                },
                "sqlite" => match config.path.as_deref().and_then(Self::resolve_path) {
                    Some(path) if path.exists() => {
                        match Connection::open_with_flags(
                            &path,
                            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                        ) {
                            Ok(_) => status = "available".to_string(),
                            Err(e) => {
                                status = "error".to_string();
                                last_error = Some(format!("SQLite read-only açılamadı: {}", e));
                            }
                        }
                    }
                    Some(path) => {
                        status = "error".to_string();
                        last_error = Some(format!("SQLite dosyası bulunamadı: {}", path.display()));
                    }
                    None => {
                        status = "error".to_string();
                        last_error = Some("SQLite path çözümlenemedi.".to_string());
                    }
                },
                "api" | "live_api" => {
                    if config
                        .base_url
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or("")
                        .is_empty()
                    {
                        status = "error".to_string();
                        last_error = Some("base_url boş.".to_string());
                    } else {
                        status = "read_only_configured".to_string();
                    }
                }
                "terminal" => {
                    status = "approval_required".to_string();
                }
                "custom_connector" => {
                    status = "disabled".to_string();
                }
                other => {
                    status = "error".to_string();
                    last_error = Some(format!("Desteklenmeyen connector tipi: {}", other));
                }
            }
        }

        SystemConnectorHealth {
            id: config.id.clone(),
            name: config.name.clone(),
            connector_type: config.connector_type.clone(),
            target,
            permissions: config.permissions.clone(),
            enabled: config.enabled,
            read_only: config.read_only_default,
            dependency_level: config.dependency_level.clone(),
            live_system: config.live_system,
            network_required: config.network_required,
            allowed_actions: config.allowed_actions.clone(),
            approval_required_actions: config.approval_required_actions.clone(),
            rollback_required_actions: config.rollback_required_actions.clone(),
            test_required_actions: config.test_required_actions.clone(),
            status,
            last_error,
            last_checked_at: Self::now_string(),
        }
    }

    fn resolve_path(path: &str) -> Option<PathBuf> {
        let root = DependencyAnalyzer::get_project_root().ok()?;
        if path.starts_with("$PROJECT_ROOT") {
            let suffix = path
                .trim_start_matches("$PROJECT_ROOT")
                .trim_start_matches('/')
                .trim_start_matches('\\');
            return Some(root.join(suffix));
        }
        if path.starts_with("$PARENT_DIR") {
            let parent = root.parent().unwrap_or(&root);
            let suffix = path
                .trim_start_matches("$PARENT_DIR")
                .trim_start_matches('/')
                .trim_start_matches('\\');
            return Some(parent.join(suffix));
        }
        if path.starts_with('.') {
            return Some(root.join(path));
        }
        Some(Path::new(path).to_path_buf())
    }

    fn audit_connector_health(result: &SystemConnectorHealth) -> Result<(), String> {
        Self::ensure_connection_audit_task()?;
        let metadata = serde_json::to_string(result).map_err(|e| e.to_string())?;
        let level = if matches!(
            result.status.as_str(),
            "available" | "disabled" | "read_only_configured" | "approval_required"
        ) {
            "info"
        } else {
            "warning"
        };
        AuditLogger::log_event(
            "__connection_audit__",
            level,
            &format!(
                "System connector health-check: {} -> {}",
                result.id, result.status
            ),
            Some("System Connector Manager"),
            Some("connector_health_check"),
            Some(&metadata),
        )
    }

    fn ensure_connection_audit_task() -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO tasks (
                id, title, user_request, status, planning_status, execution_status, risk_level, approval_status
             ) VALUES (?1, ?2, ?3, 'system', 'planning_complete', 'not_started', 'low', 'policy_checked_no_user_approval_required')",
            params![
                "__connection_audit__",
                "Bağlantı Aktivite Audit Kaydı",
                "AI provider ve sistem connector health-check aktiviteleri."
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn now_string() -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs().to_string(),
            Err(_) => "0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connector(connector_type: &str, enabled: bool) -> SystemConnectorConfig {
        SystemConnectorConfig {
            id: format!("test_{}", connector_type),
            name: "Test Connector".to_string(),
            connector_type: connector_type.to_string(),
            path: None,
            base_url: Some("https://service-domain.invalid/api".to_string()),
            api_key_env: None,
            permissions: vec!["read".to_string()],
            enabled,
            dependency_level: "low".to_string(),
            live_system: false,
            network_required: false,
            allowed_actions: vec![],
            approval_required_actions: vec!["terminal_command".to_string()],
            rollback_required_actions: vec!["terminal_command".to_string()],
            test_required_actions: vec!["terminal_command".to_string()],
            read_only_default: true,
        }
    }

    #[test]
    fn disabled_api_connector_is_not_called() {
        let result = SystemConnectorManager::health_check(&connector("api", false));
        assert_eq!(result.status, "disabled");
    }

    #[test]
    fn terminal_connector_requires_approval_without_execution() {
        let result = SystemConnectorManager::health_check(&connector("terminal", true));
        assert_eq!(result.status, "approval_required");
    }

    #[test]
    fn sqlite_connector_opens_read_only() {
        let db_path = std::env::temp_dir().join("lokal_panel_connector_health_test.db");
        let _ = std::fs::remove_file(&db_path);
        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE health_test (id INTEGER)", [])
            .unwrap();
        drop(conn);

        let mut config = connector("sqlite", true);
        config.path = Some(db_path.to_string_lossy().into_owned());
        config.base_url = None;
        config.allowed_actions = vec!["sqlite_read".to_string()];
        config.approval_required_actions = vec!["sqlite_write".to_string()];
        config.rollback_required_actions = vec!["sqlite_write".to_string()];
        config.test_required_actions = vec!["sqlite_read".to_string(), "sqlite_write".to_string()];

        let result = SystemConnectorManager::health_check(&config);
        assert_eq!(result.status, "available");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn connector_health_audit_path_records_without_external_write() {
        let results = SystemConnectorManager::health_check_all(true).unwrap();
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|connector| connector.id == "local_projects"));
    }
}

use crate::ai_providers::provider_base::{AIProviderConfig, AIProviderHealth};
use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AIProviderManager;

impl AIProviderManager {
    pub fn load_configs() -> Result<Vec<AIProviderConfig>, String> {
        let path = DependencyAnalyzer::get_config_path("ai_providers.json")?;
        let data =
            fs::read_to_string(&path).map_err(|e| format!("ai_providers.json okunamadı: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("ai_providers.json geçersiz: {}", e))
    }

    pub fn health_check_all(write_audit: bool) -> Result<Vec<AIProviderHealth>, String> {
        let configs = Self::load_configs()?;
        let mut results = Vec::new();

        for config in configs {
            let result = Self::health_check(&config);
            if write_audit {
                Self::audit_provider_health(&result)?;
            }
            results.push(result);
        }

        Ok(results)
    }

    fn health_check(config: &AIProviderConfig) -> AIProviderHealth {
        let mut status = "disabled".to_string();
        let mut api_key_status = "not_checked".to_string();
        let mut last_error = None;

        if config.enabled {
            let api_key_present = std::env::var(&config.api_key_env)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            api_key_status = if api_key_present {
                "present".to_string()
            } else {
                "missing".to_string()
            };

            if api_key_present {
                status = "available".to_string();
            } else {
                status = "missing_api_key".to_string();
                last_error = Some(format!(
                    "{} env değişkeni bulunamadı veya boş.",
                    config.api_key_env
                ));
            }
        }

        AIProviderHealth {
            id: config.id.clone(),
            name: config.name.clone(),
            provider_type: config.provider_type.clone(),
            model: config.model.clone(),
            enabled: config.enabled,
            status,
            api_key_status,
            dependency_level: config.dependency_level.clone(),
            network_required: config.network_required,
            allowed_task_types: config.allowed_task_types.clone(),
            last_error,
            last_checked_at: Self::now_string(),
        }
    }

    fn audit_provider_health(result: &AIProviderHealth) -> Result<(), String> {
        Self::ensure_connection_audit_task()?;
        let metadata = serde_json::to_string(result).map_err(|e| e.to_string())?;
        let level = if matches!(result.status.as_str(), "available" | "disabled") {
            "info"
        } else {
            "warning"
        };
        AuditLogger::log_event(
            "__connection_audit__",
            level,
            &format!(
                "AI provider health-check: {} -> {}",
                result.id, result.status
            ),
            Some("AI Provider Manager"),
            Some("provider_health_check"),
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
    use serde_json::json;

    fn provider(enabled: bool, api_key_env: &str) -> AIProviderConfig {
        AIProviderConfig {
            id: "test_provider".to_string(),
            name: "Test Provider".to_string(),
            provider_type: "openai_compatible".to_string(),
            base_url: "https://provider-domain.invalid".to_string(),
            api_key_env: api_key_env.to_string(),
            model: "test-model".to_string(),
            enabled,
            network_required: true,
            dependency_level: "high".to_string(),
            allowed_task_types: vec!["health_check".to_string()],
            max_payload_policy: json!({"max_chars": 1000}),
            sensitive_data_policy: json!({"send_secrets": false}),
        }
    }

    #[test]
    fn disabled_provider_is_not_called() {
        let result = AIProviderManager::health_check(&provider(false, "MISSING_TEST_KEY"));
        assert_eq!(result.status, "disabled");
        assert_eq!(result.api_key_status, "not_checked");
    }

    #[test]
    fn enabled_provider_without_env_reports_missing_key() {
        let result = AIProviderManager::health_check(&provider(
            true,
            "LOKAL_PANEL_INTENTIONALLY_MISSING_TEST_KEY",
        ));
        assert_eq!(result.status, "missing_api_key");
        assert_eq!(result.api_key_status, "missing");
    }

    #[test]
    fn provider_health_audit_path_does_not_call_external_api() {
        let results = AIProviderManager::health_check_all(true).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|provider| provider.status == "disabled"));
    }
}

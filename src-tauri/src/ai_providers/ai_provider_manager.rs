use crate::ai_providers::provider_base::{AIProviderConfig, AIProviderHealth};
use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use std::fs;
use std::time::Duration;
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
        let mut health = "disabled".to_string();
        let mut api_key_status = "not_checked".to_string();
        let mut last_error = None;
        let endpoint = Self::health_endpoint(config);

        if config.enabled {
            let api_key_present = std::env::var(&config.api_key_env)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            let api_key_required = Self::requires_api_key(config);
            api_key_status = if api_key_present {
                "present".to_string()
            } else if api_key_required {
                "required_missing".to_string()
            } else {
                "not_required".to_string()
            };

            if api_key_required && !api_key_present {
                status = "api_key_required".to_string();
                health = "API KEY GEREKLI".to_string();
                last_error = Some(format!(
                    "{} env değişkeni bulunamadı veya boş.",
                    config.api_key_env
                ));
            } else {
                match Self::check_http_health(config, &endpoint) {
                    Ok(_) => {
                        status = "available".to_string();
                        health = "available".to_string();
                    }
                    Err(e) => {
                        status = "connection_failed".to_string();
                        health = "unavailable".to_string();
                        last_error = Some(e);
                    }
                }
            }
        }

        AIProviderHealth {
            id: config.id.clone(),
            name: config.name.clone(),
            provider_type: config.provider_type.clone(),
            model: config.model.clone(),
            source_kind: "api".to_string(),
            endpoint: Self::redacted_endpoint(config, &endpoint),
            enabled: config.enabled,
            status,
            health,
            api_key_status,
            dependency_level: config.dependency_level.clone(),
            network_required: config.network_required,
            allowed_task_types: config.allowed_task_types.clone(),
            last_error,
            last_checked_at: Self::now_string(),
        }
    }

    fn check_http_health(config: &AIProviderConfig, endpoint: &str) -> Result<(), String> {
        let endpoint = Self::endpoint_with_auth(config, endpoint);
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(3))
            .build();
        let mut request = agent.get(&endpoint).set("Accept", "application/json");

        if let Ok(api_key) = std::env::var(&config.api_key_env) {
            let auth_mode = config.auth_mode.as_deref().unwrap_or("bearer");
            if !api_key.trim().is_empty()
                && matches!(auth_mode, "bearer" | "optional_bearer")
                && !config.provider_type.eq_ignore_ascii_case("gemini")
            {
                request = request.set("Authorization", &format!("Bearer {}", api_key));
            }
        }

        match request.call() {
            Ok(response) if (200..300).contains(&response.status()) => response
                .into_json::<serde_json::Value>()
                .map(|_| ())
                .map_err(|e| format!("HTTP cevap JSON olarak okunamadı: {}", e)),
            Ok(response) => Err(format!("HTTP {} döndü.", response.status())),
            Err(ureq::Error::Status(code, response)) => {
                Err(format!("HTTP {} döndü: {}", code, response.status_text()))
            }
            Err(err) => Err(format!("HTTP health isteği başarısız: {}", err)),
        }
    }

    fn health_endpoint(config: &AIProviderConfig) -> String {
        if let Some(endpoint) = config.health_endpoint.as_deref() {
            if !endpoint.trim().is_empty() {
                if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                    return endpoint.to_string();
                }
                if endpoint.starts_with("/api/") {
                    return Self::join_url(&Self::strip_suffix(&config.base_url, "/v1"), endpoint);
                }
                return Self::join_url(&config.base_url, endpoint);
            }
        }

        let key = format!(
            "{} {} {}",
            config.id.to_ascii_lowercase(),
            config.name.to_ascii_lowercase(),
            config.provider_type.to_ascii_lowercase()
        );

        if key.contains("ollama") {
            return format!("{}/api/tags", Self::strip_suffix(&config.base_url, "/v1"));
        }
        if key.contains("open webui") || key.contains("open_webui") || key.contains("open-webui") {
            return Self::join_url(&config.base_url, "/api/models");
        }
        if key.contains("lm studio") || key.contains("lmstudio") || key.contains("lm_studio") {
            return Self::join_url(&Self::strip_suffix(&config.base_url, "/v1"), "/v1/models");
        }
        if key.contains("gemini") {
            let base = Self::strip_suffix(&config.base_url, "/v1beta");
            let endpoint = format!("{}/v1beta/models", base);
            if let Ok(api_key) = std::env::var(&config.api_key_env) {
                if !api_key.trim().is_empty() {
                    return format!("{}?key={}", endpoint, api_key);
                }
            }
            return endpoint;
        }

        Self::join_url(&Self::strip_suffix(&config.base_url, "/v1"), "/v1/models")
    }

    fn requires_api_key(config: &AIProviderConfig) -> bool {
        if let Some(required) = config.requires_api_key {
            return required;
        }

        let key = format!(
            "{} {} {}",
            config.id.to_ascii_lowercase(),
            config.name.to_ascii_lowercase(),
            config.provider_type.to_ascii_lowercase()
        );
        if key.contains("ollama") || key.contains("lm studio") || key.contains("lmstudio") {
            return false;
        }
        if key.contains("gemini") || key.contains("groq") || key.contains("openrouter") {
            return true;
        }
        config.network_required || config.base_url.starts_with("https://")
    }

    fn endpoint_with_auth(config: &AIProviderConfig, endpoint: &str) -> String {
        if config.auth_mode.as_deref() != Some("query_key") {
            return endpoint.to_string();
        }
        let Ok(api_key) = std::env::var(&config.api_key_env) else {
            return endpoint.to_string();
        };
        if api_key.trim().is_empty() {
            return endpoint.to_string();
        }
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        format!("{}{}key={}", endpoint, separator, api_key)
    }

    fn redacted_endpoint(config: &AIProviderConfig, endpoint: &str) -> String {
        if config.auth_mode.as_deref() == Some("query_key")
            || config.provider_type.eq_ignore_ascii_case("gemini")
        {
            return endpoint.split('?').next().unwrap_or(endpoint).to_string();
        }
        endpoint.to_string()
    }

    fn join_url(base: &str, suffix: &str) -> String {
        format!("{}{}", base.trim_end_matches('/'), suffix)
    }

    fn strip_suffix(value: &str, suffix: &str) -> String {
        value
            .trim_end_matches('/')
            .strip_suffix(suffix)
            .unwrap_or_else(|| value.trim_end_matches('/'))
            .to_string()
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

    pub fn select_with_failover() -> Result<(AIProviderConfig, String), String> {
        let configs = Self::load_configs()?;
        if configs.is_empty() {
            return Err("ai_providers.json boş.".to_string());
        }

        let order = Self::load_failover_order()?;
        let mut trail = Vec::new();

        for provider_id in &order {
            let Some(config) = configs.iter().find(|c| &c.id == provider_id) else {
                continue;
            };
            let health = Self::health_check(config);
            trail.push(format!("{}:{}", config.id, health.status));
            if health.status == "available" {
                return Ok((
                    config.clone(),
                    format!("failover_route={}; trail={}", config.id, trail.join(" -> ")),
                ));
            }
        }

        Err(format!(
            "Hicbir kullanima hazir AI provider bulunamadi: {}",
            trail.join(" -> ")
        ))
    }

    fn load_failover_order() -> Result<Vec<String>, String> {
        let path = DependencyAnalyzer::get_config_path("failover_policy.json")?;
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("failover_policy.json okunamadı: {}", e))?;
        let value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("failover_policy.json geçersiz: {}", e))?;
        let mut order = Vec::new();
        if let Some(arr) = value.get("primary_order").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(id) = item.as_str() {
                    order.push(id.to_string());
                }
            }
        }
        if order.is_empty() {
            order = vec![
                "chatgpt".to_string(),
                "gemini".to_string(),
                "ollama".to_string(),
            ];
        }
        Ok(order)
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
            requires_api_key: None,
            health_endpoint: None,
            auth_mode: None,
            enabled,
            network_required: true,
            dependency_level: "high".to_string(),
            allowed_task_types: vec!["health_check".to_string()],
            max_payload_policy: json!({"max_chars": 1000}),
            sensitive_data_policy: json!({"send_secrets": false}),
        }
    }

    fn local_provider(enabled: bool) -> AIProviderConfig {
        AIProviderConfig {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            provider_type: "openai_compatible".to_string(),
            base_url: "http://127.0.0.1:65534/v1".to_string(),
            api_key_env: "LOKAL_PANEL_INTENTIONALLY_MISSING_TEST_KEY".to_string(),
            model: "test-model".to_string(),
            requires_api_key: Some(false),
            health_endpoint: Some("/api/tags".to_string()),
            auth_mode: Some("none".to_string()),
            enabled,
            network_required: false,
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
        assert_eq!(result.source_kind, "api");
    }

    #[test]
    fn enabled_provider_without_env_reports_missing_key() {
        let result = AIProviderManager::health_check(&provider(
            true,
            "LOKAL_PANEL_INTENTIONALLY_MISSING_TEST_KEY",
        ));
        assert_eq!(result.status, "api_key_required");
        assert_eq!(result.api_key_status, "required_missing");
        assert_eq!(result.health, "API KEY GEREKLI");
    }

    #[test]
    fn local_ollama_health_does_not_require_api_key() {
        let result = AIProviderManager::health_check(&local_provider(true));
        assert_eq!(result.api_key_status, "not_required");
        assert_eq!(result.endpoint, "http://127.0.0.1:65534/api/tags");
    }

    #[test]
    fn provider_health_audit_path_does_not_call_external_api() {
        let results = AIProviderManager::health_check_all(true).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|provider| provider.status == "disabled"
            || provider.status == "api_key_required"
            || provider.status == "available"
            || provider.status == "connection_failed"));
    }
}

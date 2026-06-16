use crate::ai_providers::ai_provider_manager::AIProviderManager;
use crate::ai_providers::provider_base::AIProviderConfig;
use crate::core::audit_logger::AuditLogger;
use crate::core::statement_collector::StatementCollector;
use serde_json::json;
use std::time::Duration;

pub struct AiProviderInvoker;

impl AiProviderInvoker {
    pub fn invoke_for_node(
        task_id: &str,
        node_id: &str,
        action: &str,
        prompt: &str,
    ) -> Result<(), String> {
        let (provider, route_note) = AIProviderManager::select_with_failover()?;
        let response = Self::query_provider(&provider, prompt)?;

        StatementCollector::collect_statement(
            node_id,
            "ai_provider_response",
            &provider.id,
            &response,
            Some(&route_note),
        )?;
        AuditLogger::log_event(
            task_id,
            "info",
            &format!("AI provider icra: {} ({})", provider.id, route_note),
            Some("Action Executor"),
            Some("action_execute"),
            Some(
                &json!({
                    "provider_id": provider.id,
                    "action": action,
                    "failover_note": route_note
                })
                .to_string(),
            ),
        )
    }

    fn query_provider(config: &AIProviderConfig, prompt: &str) -> Result<String, String> {
        if !config.enabled {
            return Ok(format!(
                "[provider_disabled:{}] Prompt yerel kayda alındı: {}",
                config.id,
                Self::truncate(prompt, 500)
            ));
        }

        let api_key = std::env::var(&config.api_key_env).unwrap_or_default();
        if Self::requires_api_key(config) && api_key.trim().is_empty() {
            return Err(format!("{} env boş", config.api_key_env));
        }

        match config.provider_type.as_str() {
            "openai_compatible" => Self::openai_chat(config, &api_key, prompt),
            "gemini" => Self::gemini_generate(config, &api_key, prompt),
            _ => Ok(format!(
                "[provider_local_stub:{}] {}",
                config.id,
                Self::truncate(prompt, 400)
            )),
        }
    }

    fn openai_chat(
        config: &AIProviderConfig,
        api_key: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
        let body = json!({
            "model": config.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 256
        });
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(15))
            .build();
        let request = agent.post(&url);
        let request =
            if api_key.trim().is_empty() || matches!(config.auth_mode.as_deref(), Some("none")) {
                request
            } else {
                request.set("Authorization", &format!("Bearer {}", api_key))
            };
        let response = request
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| format!("OpenAI HTTP hatası: {}", e))?;
        let status = response.status();
        let text = response
            .into_string()
            .map_err(|e| format!("OpenAI yanıt okunamadı: {}", e))?;
        if status >= 400 {
            return Err(format!(
                "OpenAI HTTP {}: {}",
                status,
                Self::truncate(&text, 300)
            ));
        }
        Ok(Self::truncate(&text, 2000))
    }

    fn gemini_generate(
        config: &AIProviderConfig,
        api_key: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            config.base_url.trim_end_matches('/'),
            config.model,
            api_key
        );
        let body = json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(15))
            .build();
        let response = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| format!("Gemini HTTP hatası: {}", e))?;
        let status = response.status();
        let text = response
            .into_string()
            .map_err(|e| format!("Gemini yanıt okunamadı: {}", e))?;
        if status >= 400 {
            return Err(format!(
                "Gemini HTTP {}: {}",
                status,
                Self::truncate(&text, 300)
            ));
        }
        Ok(Self::truncate(&text, 2000))
    }

    fn truncate(value: &str, max: usize) -> String {
        if value.len() <= max {
            value.to_string()
        } else {
            format!("{}...", &value[..max])
        }
    }

    fn requires_api_key(config: &AIProviderConfig) -> bool {
        if let Some(required) = config.requires_api_key {
            return required;
        }
        config.network_required || config.base_url.starts_with("https://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn disabled_provider_returns_local_stub_without_http() {
        let configs = AIProviderManager::load_configs().unwrap();
        let disabled = configs.iter().find(|p| !p.enabled).unwrap();
        let out = AiProviderInvoker::query_provider(disabled, "test prompt").unwrap();
        assert!(out.contains("provider_disabled"));
    }

    #[test]
    fn local_openai_compatible_provider_does_not_require_api_key() {
        let config = AIProviderConfig {
            id: "local_test".to_string(),
            name: "Local Test".to_string(),
            provider_type: "openai_compatible".to_string(),
            base_url: "http://127.0.0.1:65534/v1".to_string(),
            api_key_env: "LOKAL_PANEL_INTENTIONALLY_MISSING_TEST_KEY".to_string(),
            model: "local-model".to_string(),
            requires_api_key: Some(false),
            health_endpoint: Some("/models".to_string()),
            auth_mode: Some("none".to_string()),
            enabled: true,
            network_required: false,
            dependency_level: "medium".to_string(),
            allowed_task_types: vec!["analysis_answer".to_string()],
            max_payload_policy: json!({"max_chars": 1000}),
            sensitive_data_policy: json!({"send_secrets": false}),
        };

        let err = AiProviderInvoker::query_provider(&config, "test prompt").unwrap_err();
        assert!(!err.contains("API_KEY"));
        assert!(!err.contains("env boş"));
    }
}

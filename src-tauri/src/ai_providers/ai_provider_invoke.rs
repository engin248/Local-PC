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

        let access_type = config.access_type.clone().unwrap_or_else(|| {
            if config.network_required {
                "api_key_required".to_string()
            } else {
                "local".to_string()
            }
        });
        let api_key_required = config
            .api_key_required
            .unwrap_or_else(|| access_type != "local" && !config.api_key_env.trim().is_empty());
        let api_key = if api_key_required {
            let key = std::env::var(&config.api_key_env)
                .map_err(|_| format!("{} env tanımsız", config.api_key_env))?;
            if key.trim().is_empty() {
                return Err(format!("{} env boş", config.api_key_env));
            }
            Some(key)
        } else {
            None
        };

        match config.provider_type.as_str() {
            "openai_compatible" if config.id == "ollama" => Self::ollama_generate(config, prompt),
            "openai_compatible" | "openai_compatible_local" => {
                Self::openai_chat(config, api_key.as_deref(), prompt)
            }
            "gemini" => {
                let Some(key) = api_key.as_deref() else {
                    return Err(format!("{} env tanımsız", config.api_key_env));
                };
                Self::gemini_generate(config, key, prompt)
            }
            _ => Ok(format!(
                "[provider_local_stub:{}] {}",
                config.id,
                Self::truncate(prompt, 400)
            )),
        }
    }

    fn openai_chat(
        config: &AIProviderConfig,
        api_key: Option<&str>,
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
        let mut request = agent.post(&url).set("Content-Type", "application/json");
        if let Some(key) = api_key.filter(|key| !key.trim().is_empty()) {
            request = request.set("Authorization", &format!("Bearer {}", key));
        }
        let response = request
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

    fn ollama_generate(config: &AIProviderConfig, prompt: &str) -> Result<String, String> {
        let base_url = config
            .base_url
            .trim_end_matches('/')
            .strip_suffix("/v1")
            .unwrap_or_else(|| config.base_url.trim_end_matches('/'))
            .to_string();
        let url = format!("{}/api/generate", base_url);
        let body = json!({
            "model": config.model,
            "prompt": prompt,
            "stream": false
        });
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .build();
        let response = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| format!("Ollama HTTP hatası: {}", e))?;
        let status = response.status();
        let text = response
            .into_string()
            .map_err(|e| format!("Ollama yanıt okunamadı: {}", e))?;
        if status >= 400 {
            return Err(format!(
                "Ollama HTTP {}: {}",
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_provider_returns_local_stub_without_http() {
        let configs = AIProviderManager::load_configs().unwrap();
        let disabled = configs.iter().find(|p| !p.enabled).unwrap();
        let out = AiProviderInvoker::query_provider(disabled, "test prompt").unwrap();
        assert!(out.contains("provider_disabled"));
    }
}

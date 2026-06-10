use crate::ai_providers::provider_base::{AIProviderConfig, AIProviderHealth};
use crate::core::audit_logger::AuditLogger;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::storage::db::Database;
use rusqlite::params;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AIProviderManager;

#[derive(Debug, Clone)]
struct ProbeOutcome {
    status: String,
    api_key_status: String,
    model_list: Vec<String>,
    connection_result: String,
    last_error: Option<String>,
}

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

    pub fn test_provider(
        provider_id: &str,
        endpoint_override: Option<&str>,
    ) -> Result<AIProviderHealth, String> {
        let configs = Self::load_configs()?;
        let config = configs
            .iter()
            .find(|item| item.id == provider_id)
            .ok_or_else(|| format!("AI provider bulunamadı: {}", provider_id))?;
        let result = Self::health_check_with_endpoint(config, endpoint_override);
        Self::audit_provider_health(&result)?;
        if result.status == "available" {
            Self::save_successful_connection(&result)?;
        }
        Ok(result)
    }

    pub fn health_check(config: &AIProviderConfig) -> AIProviderHealth {
        Self::health_check_with_endpoint(config, None)
    }

    fn health_check_with_endpoint(
        config: &AIProviderConfig,
        endpoint_override: Option<&str>,
    ) -> AIProviderHealth {
        let access_type = Self::access_type(config);
        let api_key_required = Self::api_key_required(config, &access_type);
        let optional_provider = config.optional_provider.unwrap_or(false);
        let endpoint = endpoint_override
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&config.base_url)
            .to_string();

        let outcome = if !config.enabled {
            ProbeOutcome {
                status: "disabled".to_string(),
                api_key_status: "not_checked".to_string(),
                model_list: Vec::new(),
                connection_result: "Provider varsayılan kapalı.".to_string(),
                last_error: None,
            }
        } else if api_key_required && !Self::api_key_present(&config.api_key_env) {
            ProbeOutcome {
                status: if optional_provider {
                    "optional_api_key_required".to_string()
                } else {
                    "missing_api_key".to_string()
                },
                api_key_status: "missing".to_string(),
                model_list: Vec::new(),
                connection_result: format!(
                    "{} env değişkeni tanımlanınca test edilebilir.",
                    config.api_key_env
                ),
                last_error: Some(format!(
                    "{} env değişkeni bulunamadı veya boş.",
                    config.api_key_env
                )),
            }
        } else {
            let mut result = Self::probe_provider(config, &endpoint, &access_type);
            if api_key_required {
                result.api_key_status = "present".to_string();
            } else {
                result.api_key_status = "not_required".to_string();
            }
            result
        };

        AIProviderHealth {
            id: config.id.clone(),
            name: config.name.clone(),
            provider_type: config.provider_type.clone(),
            model: config.model.clone(),
            endpoint,
            enabled: config.enabled,
            status: outcome.status,
            api_key_status: outcome.api_key_status,
            access_type,
            api_key_required,
            optional_provider,
            dependency_level: config.dependency_level.clone(),
            network_required: config.network_required,
            allowed_task_types: config.allowed_task_types.clone(),
            model_list: outcome.model_list,
            connection_result: outcome.connection_result,
            error_message: outcome.last_error.clone(),
            last_error: outcome.last_error,
            last_checked_at: Self::now_string(),
        }
    }

    fn access_type(config: &AIProviderConfig) -> String {
        if let Some(access_type) = &config.access_type {
            return access_type.clone();
        }
        if !config.network_required {
            "local".to_string()
        } else if !config.api_key_env.trim().is_empty() {
            "api_key_required".to_string()
        } else {
            "free_tier".to_string()
        }
    }

    fn api_key_required(config: &AIProviderConfig, access_type: &str) -> bool {
        config
            .api_key_required
            .unwrap_or_else(|| access_type != "local" && !config.api_key_env.trim().is_empty())
    }

    fn api_key_present(env_name: &str) -> bool {
        if env_name.trim().is_empty() {
            return false;
        }
        std::env::var(env_name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }

    fn probe_provider(
        config: &AIProviderConfig,
        endpoint: &str,
        access_type: &str,
    ) -> ProbeOutcome {
        match config.id.as_str() {
            "ollama" => Self::probe_ollama(config, endpoint),
            "open_webui" => Self::probe_open_webui(config, endpoint),
            "huggingface_local" => Self::probe_huggingface_cache(),
            "pinokio" => Self::probe_pinokio(config),
            "lm_studio" => Self::probe_openai_models(endpoint, "LM Studio"),
            "openai_compatible_local" => {
                Self::probe_openai_models(endpoint, "OpenAI-compatible local")
            }
            _ if access_type == "local" => match Self::check_tcp_connection(endpoint, 2) {
                Ok(_) => ProbeOutcome {
                    status: "available".to_string(),
                    api_key_status: "not_required".to_string(),
                    model_list: vec![config.model.clone()],
                    connection_result: "Lokal TCP endpoint erişilebilir.".to_string(),
                    last_error: None,
                },
                Err(error) => ProbeOutcome {
                    status: "connection_failed".to_string(),
                    api_key_status: "not_required".to_string(),
                    model_list: Vec::new(),
                    connection_result: "Lokal TCP endpoint yanıt vermedi.".to_string(),
                    last_error: Some(error),
                },
            },
            _ => match Self::check_tcp_connection(endpoint, 3) {
                Ok(_) => ProbeOutcome {
                    status: "available".to_string(),
                    api_key_status: "present".to_string(),
                    model_list: vec![config.model.clone()],
                    connection_result:
                        "Endpoint TCP seviyesinde erişilebilir; canlı içerik çağrısı yapılmadı."
                            .to_string(),
                    last_error: None,
                },
                Err(error) => ProbeOutcome {
                    status: "connection_failed".to_string(),
                    api_key_status: "present".to_string(),
                    model_list: Vec::new(),
                    connection_result: "Endpoint TCP seviyesinde erişilemedi.".to_string(),
                    last_error: Some(error),
                },
            },
        }
    }

    fn probe_ollama(config: &AIProviderConfig, endpoint: &str) -> ProbeOutcome {
        let base = Self::ollama_base_url(endpoint);
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(5))
            .build();
        let tags_url = format!("{}/api/tags", base.trim_end_matches('/'));
        let tags = match Self::http_get_json(&agent, &tags_url) {
            Ok(value) => value,
            Err(error) => {
                return ProbeOutcome {
                    status: "connection_failed".to_string(),
                    api_key_status: "not_required".to_string(),
                    model_list: Vec::new(),
                    connection_result: format!("Ollama /api/tags erişilemedi: {}", error),
                    last_error: Some(error),
                };
            }
        };

        let models = tags
            .get("models")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("name").and_then(|name| name.as_str()))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if models.is_empty() {
            return ProbeOutcome {
                status: "available_no_models".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: Vec::new(),
                connection_result: "Ollama çalışıyor; /api/tags model döndürmedi.".to_string(),
                last_error: None,
            };
        }

        let selected_model = if models.iter().any(|model| model == &config.model) {
            config.model.clone()
        } else {
            models[0].clone()
        };
        let generate_url = format!("{}/api/generate", base.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": selected_model,
            "prompt": "health",
            "stream": false,
            "options": { "num_predict": 1 }
        });
        match Self::http_post_json(&agent, &generate_url, body) {
            Ok(_) => ProbeOutcome {
                status: "available".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: models,
                connection_result: "Ollama /api/tags ve /api/generate başarılı.".to_string(),
                last_error: None,
            },
            Err(error) => ProbeOutcome {
                status: "test_failed".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: models,
                connection_result: "Ollama /api/tags başarılı, /api/generate başarısız."
                    .to_string(),
                last_error: Some(error),
            },
        }
    }

    fn ollama_base_url(endpoint: &str) -> String {
        endpoint
            .trim_end_matches('/')
            .strip_suffix("/v1")
            .unwrap_or_else(|| endpoint.trim_end_matches('/'))
            .to_string()
    }

    fn probe_open_webui(config: &AIProviderConfig, endpoint: &str) -> ProbeOutcome {
        let candidates = Self::endpoint_candidates(endpoint, &config.discovery_ports);
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(3))
            .build();

        for candidate in candidates {
            match Self::http_get_text(&agent, &candidate) {
                Ok(_) => {
                    let models_url = format!("{}/api/models", candidate.trim_end_matches('/'));
                    let models = Self::http_get_json(&agent, &models_url)
                        .ok()
                        .and_then(|value| Self::extract_model_ids(&value))
                        .unwrap_or_default();
                    return ProbeOutcome {
                        status: "available".to_string(),
                        api_key_status: "not_required".to_string(),
                        model_list: models,
                        connection_result: format!("Open WebUI endpoint keşfedildi: {}", candidate),
                        last_error: None,
                    };
                }
                Err(_) => continue,
            }
        }

        ProbeOutcome {
            status: "connection_failed".to_string(),
            api_key_status: "not_required".to_string(),
            model_list: Vec::new(),
            connection_result: "Open WebUI local endpoint discovery başarısız.".to_string(),
            last_error: Some("Aday Open WebUI portları yanıt vermedi.".to_string()),
        }
    }

    fn probe_openai_models(endpoint: &str, label: &str) -> ProbeOutcome {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(3))
            .build();
        let url = format!("{}/models", endpoint.trim_end_matches('/'));
        match Self::http_get_json(&agent, &url) {
            Ok(value) => {
                let models = Self::extract_model_ids(&value).unwrap_or_default();
                ProbeOutcome {
                    status: "available".to_string(),
                    api_key_status: "not_required".to_string(),
                    model_list: models,
                    connection_result: format!("{} /models endpoint başarılı.", label),
                    last_error: None,
                }
            }
            Err(error) => ProbeOutcome {
                status: "connection_failed".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: Vec::new(),
                connection_result: format!("{} /models endpoint başarısız.", label),
                last_error: Some(error),
            },
        }
    }

    fn probe_huggingface_cache() -> ProbeOutcome {
        let mut roots = Vec::new();
        if let Some(home) = std::env::var_os("HF_HOME") {
            roots.push(PathBuf::from(home).join("hub"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            roots.push(home.join(".cache/huggingface/hub"));
            roots.push(home.join(".cache/huggingface/transformers"));
        }

        let mut models = Vec::new();
        let mut checked = Vec::new();
        for root in roots {
            checked.push(root.to_string_lossy().into_owned());
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with("models--") || entry.path().is_dir() {
                        models.push(name);
                    }
                }
            }
        }
        models.sort();
        models.dedup();

        if models.is_empty() {
            ProbeOutcome {
                status: "not_found".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: Vec::new(),
                connection_result: format!(
                    "HuggingFace local cache/model klasörü bulunamadı. Kontrol: {}",
                    checked.join(", ")
                ),
                last_error: Some("HuggingFace cache boş veya yok.".to_string()),
            }
        } else {
            ProbeOutcome {
                status: "available".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: models,
                connection_result: "HuggingFace local model cache bulundu.".to_string(),
                last_error: None,
            }
        }
    }

    fn probe_pinokio(config: &AIProviderConfig) -> ProbeOutcome {
        let ports = if config.discovery_ports.is_empty() {
            vec![42000, 7860, 7861, 7862, 8000, 8080, 8188]
        } else {
            config.discovery_ports.clone()
        };
        let open_ports = ports
            .into_iter()
            .filter(|port| Self::tcp_port_open("127.0.0.1", *port, 1))
            .map(|port| format!("127.0.0.1:{}", port))
            .collect::<Vec<_>>();
        let processes = Self::process_hits(&["pinokio", "gradio", "comfyui", "stable-diffusion"]);

        if !open_ports.is_empty() || !processes.is_empty() {
            let mut models = open_ports.clone();
            models.extend(processes.iter().take(5).cloned());
            ProbeOutcome {
                status: "available".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: models,
                connection_result: "Pinokio benzeri local AI port/process izi bulundu.".to_string(),
                last_error: None,
            }
        } else {
            ProbeOutcome {
                status: "not_found".to_string(),
                api_key_status: "not_required".to_string(),
                model_list: Vec::new(),
                connection_result:
                    "Pinokio port/process taramasında çalışan AI uygulaması bulunmadı.".to_string(),
                last_error: Some("Pinokio servis izi yok.".to_string()),
            }
        }
    }

    fn endpoint_candidates(endpoint: &str, ports: &[u16]) -> Vec<String> {
        let mut candidates = vec![endpoint.trim_end_matches('/').to_string()];
        for port in ports {
            candidates.push(format!("http://127.0.0.1:{}", port));
        }
        candidates.sort();
        candidates.dedup();
        candidates
    }

    fn extract_model_ids(value: &serde_json::Value) -> Option<Vec<String>> {
        let array = value
            .get("data")
            .or_else(|| value.get("models"))
            .and_then(|value| value.as_array())?;
        Some(
            array
                .iter()
                .filter_map(|item| {
                    item.get("id")
                        .or_else(|| item.get("name"))
                        .and_then(|value| value.as_str())
                })
                .map(str::to_string)
                .collect(),
        )
    }

    fn http_get_json(agent: &ureq::Agent, url: &str) -> Result<serde_json::Value, String> {
        let text = Self::http_get_text(agent, url)?;
        serde_json::from_str(&text).map_err(|error| format!("JSON okunamadı: {}", error))
    }

    fn http_get_text(agent: &ureq::Agent, url: &str) -> Result<String, String> {
        match agent.get(url).call() {
            Ok(response) => response
                .into_string()
                .map_err(|error| format!("HTTP yanıtı okunamadı: {}", error)),
            Err(ureq::Error::Status(status, response)) => {
                let text = response.into_string().unwrap_or_default();
                Err(format!("HTTP {}: {}", status, Self::truncate(&text, 200)))
            }
            Err(error) => Err(format!("{}", error)),
        }
    }

    fn http_post_json(
        agent: &ureq::Agent,
        url: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match agent
            .post(url)
            .set("Content-Type", "application/json")
            .send_json(body)
        {
            Ok(response) => response
                .into_json()
                .map_err(|error| format!("HTTP JSON yanıtı okunamadı: {}", error)),
            Err(ureq::Error::Status(status, response)) => {
                let text = response.into_string().unwrap_or_default();
                Err(format!("HTTP {}: {}", status, Self::truncate(&text, 200)))
            }
            Err(error) => Err(format!("{}", error)),
        }
    }

    fn check_tcp_connection(url_str: &str, timeout_secs: u64) -> Result<(), String> {
        let without_protocol = if let Some(stripped) = url_str.strip_prefix("https://") {
            stripped
        } else if let Some(stripped) = url_str.strip_prefix("http://") {
            stripped
        } else {
            url_str
        };

        let authority = without_protocol
            .split('/')
            .next()
            .unwrap_or(without_protocol);

        let (host, port) = if let Some(pos) = authority.find(':') {
            let (h, p) = authority.split_at(pos);
            (h, p.trim_start_matches(':'))
        } else {
            if url_str.starts_with("https://") {
                (authority, "443")
            } else {
                (authority, "80")
            }
        };

        let addr_str = format!("{}:{}", host, port);
        let socket_addrs = addr_str
            .to_socket_addrs()
            .map_err(|e| format!("Adres çözümlenemedi ({}): {}", addr_str, e))?;

        let mut success = false;
        let mut last_err = "Host adresi bulunamadı".to_string();

        for addr in socket_addrs {
            match TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)) {
                Ok(_) => {
                    success = true;
                    break;
                }
                Err(e) => {
                    last_err = format!(
                        "Bağlantı zaman aşımına uğradı veya reddedildi ({}): {}",
                        addr, e
                    );
                }
            }
        }

        if success {
            Ok(())
        } else {
            Err(last_err)
        }
    }

    fn tcp_port_open(host: &str, port: u16, timeout_secs: u64) -> bool {
        let addr_str = format!("{}:{}", host, port);
        let Ok(socket_addrs) = addr_str.to_socket_addrs() else {
            return false;
        };
        socket_addrs.into_iter().any(|addr| {
            TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)).is_ok()
        })
    }

    fn process_hits(keywords: &[&str]) -> Vec<String> {
        #[cfg(target_os = "windows")]
        let output = Command::new("wmic")
            .args(["process", "get", "ProcessId,CommandLine"])
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("ps").args(["-eo", "pid,comm,args"]).output();

        let Ok(output) = output else {
            return Vec::new();
        };
        let text = String::from_utf8_lossy(&output.stdout).to_lowercase();
        text.lines()
            .filter(|line| keywords.iter().any(|keyword| line.contains(keyword)))
            .take(20)
            .map(|line| line.chars().take(180).collect())
            .collect()
    }

    fn save_successful_connection(result: &AIProviderHealth) -> Result<(), String> {
        let path = DependencyAnalyzer::get_config_path("ai_providers.json")?;
        let data =
            fs::read_to_string(&path).map_err(|e| format!("ai_providers.json okunamadı: {}", e))?;
        let mut value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("ai_providers.json geçersiz: {}", e))?;
        let Some(items) = value.as_array_mut() else {
            return Err("ai_providers.json dizi formatında değil.".to_string());
        };
        for item in items {
            if item.get("id").and_then(|id| id.as_str()) == Some(result.id.as_str()) {
                item["base_url"] = serde_json::Value::String(result.endpoint.clone());
                item["last_success_at"] = serde_json::Value::String(result.last_checked_at.clone());
                item["last_status"] = serde_json::Value::String(result.status.clone());
                item["discovered_models"] = serde_json::Value::Array(
                    result
                        .model_list
                        .iter()
                        .cloned()
                        .map(serde_json::Value::String)
                        .collect(),
                );
                break;
            }
        }
        let next =
            serde_json::to_string_pretty(&value).map_err(|e| format!("JSON yazılamadı: {}", e))?;
        fs::write(path, format!("{}\n", next))
            .map_err(|e| format!("ai_providers.json kaydedilemedi: {}", e))
    }

    fn audit_provider_health(result: &AIProviderHealth) -> Result<(), String> {
        Self::ensure_connection_audit_task()?;
        let metadata = serde_json::to_string(result).map_err(|e| e.to_string())?;
        let level = if matches!(
            result.status.as_str(),
            "available" | "disabled" | "optional_api_key_required"
        ) {
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

    fn truncate(value: &str, max: usize) -> String {
        if value.len() <= max {
            value.to_string()
        } else {
            format!("{}...", &value[..max])
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
            enabled,
            network_required: true,
            dependency_level: "high".to_string(),
            allowed_task_types: vec!["health_check".to_string()],
            max_payload_policy: json!({"max_chars": 1000}),
            sensitive_data_policy: json!({"send_secrets": false}),
            access_type: None,
            api_key_required: None,
            optional_provider: None,
            discovery_ports: Vec::new(),
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
        assert!(results.iter().all(|provider| matches!(
            provider.status.as_str(),
            "disabled"
                | "missing_api_key"
                | "optional_api_key_required"
                | "available"
                | "available_no_models"
                | "connection_failed"
                | "not_found"
                | "test_failed"
        )));
    }

    #[test]
    fn local_provider_does_not_require_api_key() {
        let mut config = provider(true, "");
        config.id = "local_stub".to_string();
        config.network_required = false;
        config.access_type = Some("local".to_string());
        config.base_url = "http://127.0.0.1:9".to_string();

        let result = AIProviderManager::health_check(&config);
        assert_eq!(result.api_key_status, "not_required");
        assert_eq!(result.api_key_required, false);
        assert_eq!(result.access_type, "local");
    }

    #[test]
    fn optional_free_tier_provider_reports_key_requirement_separately() {
        let mut config = provider(true, "LOKAL_PANEL_INTENTIONALLY_MISSING_FREE_KEY");
        config.id = "groq".to_string();
        config.access_type = Some("free_tier".to_string());
        config.api_key_required = Some(true);
        config.optional_provider = Some(true);

        let result = AIProviderManager::health_check(&config);
        assert_eq!(result.status, "optional_api_key_required");
        assert_eq!(result.api_key_status, "missing");
        assert!(result.optional_provider);
    }
}

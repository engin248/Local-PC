use super::{push_error, validate_no_production_placeholder};
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn validate_ai_providers(
    providers: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let providers = providers
        .as_array()
        .ok_or_else(|| "ai_providers.json liste formatında olmalıdır.".to_string())?;

    let mut ids = HashSet::new();
    for provider in providers {
        let id = provider
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "AI provider id alanı eksik.".to_string())?;
        validate_no_production_placeholder(
            "AI_PROVIDER_PRODUCTION_PLACEHOLDER",
            id,
            provider,
            issues,
        );
        if !ids.insert(id.to_string()) {
            push_error(
                issues,
                "AI_PROVIDER_DUPLICATE_ID",
                format!("Tekrarlanan AI provider id: {}", id),
            );
        }
        let provider_type = provider
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("AI provider type alanı eksik: {}", id))?;
        if !matches!(
            provider_type,
            "openai_compatible" | "gemini" | "perplexity" | "verdent" | "custom_api"
        ) {
            push_error(
                issues,
                "AI_PROVIDER_TYPE_INVALID",
                format!("AI provider type geçersiz: {} -> {}", id, provider_type),
            );
        }

        let enabled = provider
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| format!("AI provider enabled alanı eksik: {}", id))?;
        let dependency_level = provider
            .get("dependency_level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("AI provider dependency_level alanı eksik: {}", id))?;
        if !matches!(dependency_level, "low" | "medium" | "high" | "critical") {
            push_error(
                issues,
                "AI_PROVIDER_LEVEL_INVALID",
                format!(
                    "AI provider dependency_level geçersiz: {} -> {}",
                    id, dependency_level
                ),
            );
        }
        if provider
            .get("base_url")
            .and_then(|v| v.as_str())
            .map(|v| v.trim().is_empty())
            .unwrap_or(true)
            && provider_type != "custom_api"
        {
            push_error(
                issues,
                "AI_PROVIDER_BASE_URL_MISSING",
                format!("AI provider base_url boş: {}", id),
            );
        }
        if provider
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.trim().is_empty())
            .unwrap_or(true)
        {
            push_error(
                issues,
                "AI_PROVIDER_MODEL_MISSING",
                format!("AI provider model alanı boş: {}", id),
            );
        }
        if enabled
            && provider
                .get("api_key_env")
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
        {
            push_error(
                issues,
                "AI_PROVIDER_API_KEY_ENV_MISSING",
                format!("Etkin AI provider api_key_env olmadan çalışamaz: {}", id),
            );
        }
        if provider
            .get("network_required")
            .and_then(|v| v.as_bool())
            .is_none()
        {
            push_error(
                issues,
                "AI_PROVIDER_NETWORK_REQUIRED_MISSING",
                format!("AI provider network_required alanı eksik: {}", id),
            );
        }
        if provider
            .get("allowed_task_types")
            .and_then(|v| v.as_array())
            .map(|v| v.is_empty())
            .unwrap_or(true)
        {
            push_error(
                issues,
                "AI_PROVIDER_ALLOWED_TASKS_MISSING",
                format!("AI provider allowed_task_types boş veya eksik: {}", id),
            );
        }
        if provider.get("sensitive_data_policy").is_none() {
            push_error(
                issues,
                "AI_PROVIDER_SENSITIVE_POLICY_MISSING",
                format!("AI provider sensitive_data_policy eksik: {}", id),
            );
        }
        if provider.get("max_payload_policy").is_none() {
            push_error(
                issues,
                "AI_PROVIDER_PAYLOAD_POLICY_MISSING",
                format!("AI provider max_payload_policy eksik: {}", id),
            );
        }
    }

    Ok(())
}

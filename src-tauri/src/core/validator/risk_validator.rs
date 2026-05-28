use super::{is_write_like, push_error};
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn validate_risk_rules(
    risk: &Value,
    approval_actions: &HashSet<String>,
    rollback_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let levels = risk
        .get("levels")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "risk_rules.json içinde levels bulunamadı.".to_string())?;
    for required in ["low", "medium", "high", "critical"] {
        if !levels.contains_key(required) {
            push_error(
                issues,
                "RISK_LEVEL_MISSING",
                format!("Risk seviyesi eksik: {}", required),
            );
        }
    }

    let mappings = risk
        .get("action_mappings")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "risk_rules.json içinde action_mappings bulunamadı.".to_string())?;

    for (action, config) in mappings {
        let level = config.get("level").and_then(|v| v.as_str()).unwrap_or("");
        if !matches!(level, "low" | "medium" | "high" | "critical") {
            push_error(
                issues,
                "RISK_ACTION_LEVEL_INVALID",
                format!("Aksiyon risk seviyesi geçersiz: {} -> {}", action, level),
            );
        }
        for field in ["reason", "mitigation"] {
            if config
                .get(field)
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
            {
                push_error(
                    issues,
                    "RISK_ACTION_TEXT_MISSING",
                    format!("{} için {} alanı boş veya eksik.", action, field),
                );
            }
        }
        if config.get("assets").and_then(|v| v.as_array()).is_none() {
            push_error(
                issues,
                "RISK_ACTION_ASSETS_MISSING",
                format!("{} için assets listesi eksik.", action),
            );
        }

        if matches!(level, "high" | "critical") && !approval_actions.contains(action) {
            push_error(
                issues,
                "HIGH_RISK_APPROVAL_MISSING",
                format!(
                    "Yüksek/kritik aksiyon approval_rules içinde yok: {}",
                    action
                ),
            );
        }
        if is_write_like(action) && !rollback_actions.contains(action) {
            push_error(
                issues,
                "WRITE_ROLLBACK_MISSING",
                format!("Yan etkili aksiyon rollback_rules içinde yok: {}", action),
            );
        }
    }

    Ok(())
}

use super::push_error;
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn validate_authority_matrix(
    authority: &Value,
    known_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let authority_obj = authority
        .as_object()
        .ok_or_else(|| "authority_matrix.json nesne formatında olmalıdır.".to_string())?;

    for (action, deciders) in authority_obj {
        if action == "level_mappings" {
            continue;
        }
        if !known_actions.contains(action) {
            push_error(
                issues,
                "AUTHORITY_UNKNOWN_ACTION",
                format!(
                    "Yetki matrisi risk kurallarında olmayan aksiyon içeriyor: {}",
                    action
                ),
            );
        }
        if deciders.as_array().map(|v| v.is_empty()).unwrap_or(true) {
            push_error(
                issues,
                "AUTHORITY_EMPTY_DECIDERS",
                format!("Aksiyon için yetkili karar verici listesi boş: {}", action),
            );
        }
    }

    let level_mappings = authority
        .get("level_mappings")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "authority_matrix.json içinde level_mappings bulunamadı.".to_string())?;
    for (level, action) in level_mappings {
        let action = action
            .as_str()
            .ok_or_else(|| format!("level_mappings.{} string olmalıdır.", level))?;
        if !known_actions.contains(action) {
            push_error(
                issues,
                "LEVEL_MAPPING_UNKNOWN_ACTION",
                format!(
                    "Seviye eşlemesi risk kurallarında olmayan aksiyona gidiyor: {} -> {}",
                    level, action
                ),
            );
        }
    }

    Ok(())
}

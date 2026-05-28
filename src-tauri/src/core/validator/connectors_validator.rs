use super::{is_write_like, optional_string_set, push_error, validate_no_production_placeholder};
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn validate_connectors(
    connectors: &Value,
    known_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let connectors = connectors
        .as_array()
        .ok_or_else(|| "system_connectors.json liste formatında olmalıdır.".to_string())?;

    let mut ids = HashSet::new();
    for connector in connectors {
        let id = connector
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Connector id alanı eksik.".to_string())?;
        validate_no_production_placeholder(
            "CONNECTOR_PRODUCTION_PLACEHOLDER",
            id,
            connector,
            issues,
        );
        if !ids.insert(id.to_string()) {
            push_error(
                issues,
                "CONNECTOR_DUPLICATE_ID",
                format!("Tekrarlanan connector id: {}", id),
            );
        }
        let connector_type = connector
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Connector type alanı eksik: {}", id))?;
        if !matches!(
            connector_type,
            "folder" | "file" | "sqlite" | "api" | "live_api" | "terminal" | "custom_connector"
        ) {
            push_error(
                issues,
                "CONNECTOR_TYPE_INVALID",
                format!("Connector type geçersiz: {} -> {}", id, connector_type),
            );
        }
        let enabled = connector
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| format!("Connector enabled alanı eksik: {}", id))?;
        let dependency_level = connector
            .get("dependency_level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Connector dependency_level alanı eksik: {}", id))?;

        if !matches!(dependency_level, "low" | "medium" | "high" | "critical") {
            push_error(
                issues,
                "CONNECTOR_LEVEL_INVALID",
                format!(
                    "Connector dependency_level geçersiz: {} -> {}",
                    id, dependency_level
                ),
            );
        }
        if matches!(connector_type, "folder" | "file" | "sqlite")
            && connector
                .get("path")
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
        {
            push_error(
                issues,
                "CONNECTOR_PATH_MISSING",
                format!("{} connector path alanı olmadan kullanılamaz.", id),
            );
        }
        if matches!(connector_type, "api" | "live_api") {
            let base_url_empty = connector
                .get("base_url")
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true);
            if enabled && base_url_empty {
                push_error(
                    issues,
                    "CONNECTOR_API_BASE_URL_MISSING",
                    format!("Etkin API connector base_url olmadan çalışamaz: {}", id),
                );
            }
        }

        for field in [
            "permissions",
            "allowed_actions",
            "approval_required_actions",
            "rollback_required_actions",
            "test_required_actions",
        ] {
            if connector.get(field).and_then(|v| v.as_array()).is_none() {
                push_error(
                    issues,
                    "CONNECTOR_ACTION_LIST_MISSING",
                    format!("{} connector {} alanı liste olmalıdır.", id, field),
                );
            }
        }

        if connector.get("read_only_default").and_then(|v| v.as_bool()) != Some(true) {
            push_error(
                issues,
                "CONNECTOR_READ_ONLY_DEFAULT_MISSING",
                format!("{} connector read_only_default=true olmalıdır.", id),
            );
        }

        let permissions = optional_string_set(connector, "permissions");
        for permission in &permissions {
            if !matches!(
                permission.as_str(),
                "read"
                    | "write_with_approval"
                    | "delete_with_approval"
                    | "execute_with_approval"
                    | "api_write_with_approval"
                    | "db_write_with_approval"
            ) {
                push_error(
                    issues,
                    "CONNECTOR_PERMISSION_INVALID",
                    format!("{} connector izni geçersiz: {}", id, permission),
                );
            }
        }

        let allowed_actions = optional_string_set(connector, "allowed_actions");
        let approval_required_actions = optional_string_set(connector, "approval_required_actions");
        let rollback_required_actions = optional_string_set(connector, "rollback_required_actions");
        let test_required_actions = optional_string_set(connector, "test_required_actions");

        for action in allowed_actions
            .iter()
            .chain(approval_required_actions.iter())
            .chain(rollback_required_actions.iter())
            .chain(test_required_actions.iter())
        {
            if !known_actions.contains(action) {
                push_error(
                    issues,
                    "CONNECTOR_UNKNOWN_ACTION",
                    format!("{} connector bilinmeyen aksiyon içeriyor: {}", id, action),
                );
            }
        }

        let risky_actions: HashSet<String> = allowed_actions
            .iter()
            .chain(approval_required_actions.iter())
            .filter(|action| is_write_like(action))
            .cloned()
            .collect();
        for action in risky_actions {
            if !approval_required_actions.contains(&action) {
                push_error(
                    issues,
                    "CONNECTOR_APPROVAL_ACTION_MISSING",
                    format!("{} riskli aksiyon approval listesinde yok: {}", id, action),
                );
            }
            if !rollback_required_actions.contains(&action) {
                push_error(
                    issues,
                    "CONNECTOR_ROLLBACK_ACTION_MISSING",
                    format!("{} riskli aksiyon rollback listesinde yok: {}", id, action),
                );
            }
            if !test_required_actions.contains(&action) {
                push_error(
                    issues,
                    "CONNECTOR_TEST_ACTION_MISSING",
                    format!("{} riskli aksiyon test listesinde yok: {}", id, action),
                );
            }
        }
    }

    Ok(())
}

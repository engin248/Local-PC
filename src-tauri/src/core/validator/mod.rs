use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub mod approval_validator;
pub mod authority_validator;
pub mod connectors_validator;
pub mod planning_validator;
pub mod providers_validator;
pub mod risk_validator;
pub mod rollback_validator;

pub(crate) fn push_error(
    issues: &mut Vec<SystemValidationIssue>,
    code: &str,
    message: impl Into<String>,
) {
    issues.push(SystemValidationIssue {
        severity: "error".to_string(),
        code: code.to_string(),
        message: message.into(),
    });
}

pub(crate) fn validate_action_set(
    source: &str,
    actions: &HashSet<String>,
    known_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) {
    for action in actions {
        if !known_actions.contains(action) {
            push_error(
                issues,
                "POLICY_UNKNOWN_ACTION",
                format!("{} bilinmeyen aksiyon içeriyor: {}", source, action),
            );
        }
    }
}

pub(crate) fn validate_no_production_placeholder(
    code: &str,
    id: &str,
    item: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) {
    if item.get("placeholder").and_then(|v| v.as_bool()) == Some(true) {
        push_error(
            issues,
            code,
            format!("Production config placeholder kaydı içeremez: {}", id),
        );
    }

    if contains_forbidden_production_text(item) {
        push_error(
            issues,
            code,
            format!(
                "Production config şablon/example/demo/mock metni içeremez: {}",
                id
            ),
        );
    }
}

pub(crate) fn contains_forbidden_production_text(value: &Value) -> bool {
    match value {
        Value::String(text) => {
            let lower = text.to_lowercase();
            lower.contains("example.com")
                || lower.contains("_template")
                || lower.contains(" template")
                || lower.contains("şablon")
                || lower.contains("sablon")
                || lower.contains("mock")
                || lower.contains("demo")
                || lower.contains("fake")
                || lower.contains("sahte")
        }
        Value::Array(items) => items.iter().any(contains_forbidden_production_text),
        Value::Object(map) => map.values().any(contains_forbidden_production_text),
        _ => false,
    }
}

pub(crate) fn optional_string_set(value: &Value, field: &str) -> HashSet<String> {
    match value.get(field).and_then(|v| v.as_array()) {
        Some(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|text| text.to_string()))
            .collect(),
        None => HashSet::new(),
    }
}

pub(crate) fn is_write_like(action: &str) -> bool {
    action.contains("write")
        || action.contains("delete")
        || action == "terminal_command"
        || action == "live_system_update"
}

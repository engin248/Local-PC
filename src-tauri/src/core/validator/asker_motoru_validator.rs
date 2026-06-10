use super::{push_error, validate_no_production_placeholder};
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn validate_asker_motoru(
    config: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    validate_no_production_placeholder(
        "ASKER_MOTORU_PRODUCTION_PLACEHOLDER",
        "asker_motoru",
        config,
        issues,
    );

    if config
        .get("schema_version")
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
        == 0
    {
        push_error(
            issues,
            "ASKER_MOTORU_SCHEMA_VERSION_MISSING",
            "asker_motoru.json schema_version alanı pozitif sayı olmalıdır.",
        );
    }

    validate_roots(config, issues)?;
    validate_status_files(config, issues)?;
    validate_api_contract(config, issues)?;
    Ok(())
}

fn validate_roots(config: &Value, issues: &mut Vec<SystemValidationIssue>) -> Result<(), String> {
    let roots = config
        .get("roots")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "asker_motoru.json roots alanı liste olmalıdır.".to_string())?;
    if roots.is_empty() {
        push_error(
            issues,
            "ASKER_MOTORU_ROOTS_EMPTY",
            "asker_motoru.json en az bir root tanımlamalıdır.",
        );
    }

    let mut ids = HashSet::new();
    for root in roots {
        let id = required_string(root, "id", "root")?;
        if !ids.insert(id.to_string()) {
            push_error(
                issues,
                "ASKER_MOTORU_ROOT_DUPLICATE",
                format!("Tekrarlanan Asker Motoru root id: {}", id),
            );
        }
        for field in ["path", "role"] {
            let _ = required_string(root, field, id)?;
        }
    }

    Ok(())
}

fn validate_status_files(
    config: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let status_files = config
        .get("status_files")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "asker_motoru.json status_files alanı liste olmalıdır.".to_string())?;
    if status_files.is_empty() {
        push_error(
            issues,
            "ASKER_MOTORU_STATUS_FILES_EMPTY",
            "asker_motoru.json en az bir status dosyası tanımlamalıdır.",
        );
    }
    for item in status_files {
        if item
            .as_str()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            push_error(
                issues,
                "ASKER_MOTORU_STATUS_FILE_INVALID",
                "asker_motoru.json status_files içinde boş olmayan string beklenir.",
            );
        }
    }

    Ok(())
}

fn validate_api_contract(
    config: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let api = config
        .get("api")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "asker_motoru.json api alanı nesne olmalıdır.".to_string())?;
    if api
        .get("enabled")
        .and_then(|value| value.as_bool())
        .is_none()
    {
        push_error(
            issues,
            "ASKER_MOTORU_API_ENABLED_MISSING",
            "asker_motoru.json api.enabled boolean olmalıdır.",
        );
    }
    if api
        .get("base_url")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        push_error(
            issues,
            "ASKER_MOTORU_API_BASE_URL_MISSING",
            "asker_motoru.json api.base_url boş olamaz.",
        );
    }

    let endpoints = api
        .get("endpoints")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "asker_motoru.json api.endpoints alanı nesne olmalıdır.".to_string())?;
    for (name, method, path) in [
        ("health", "GET", "/health"),
        ("status", "GET", "/status"),
        ("events", "GET", "/events"),
        ("command", "POST", "/command"),
    ] {
        let endpoint = endpoints
            .get(name)
            .and_then(|value| value.as_object())
            .ok_or_else(|| format!("Asker Motoru endpoint eksik: {}", name))?;
        let actual_method = endpoint
            .get("method")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if actual_method != method {
            push_error(
                issues,
                "ASKER_MOTORU_ENDPOINT_METHOD_INVALID",
                format!("{} endpoint method {} olmalıdır.", name, method),
            );
        }
        let actual_path = endpoint
            .get("path")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if actual_path != path {
            push_error(
                issues,
                "ASKER_MOTORU_ENDPOINT_PATH_INVALID",
                format!("{} endpoint path {} olmalıdır.", name, path),
            );
        }
    }

    Ok(())
}

fn required_string<'a>(value: &'a Value, field: &str, owner: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(|item| item.as_str())
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("Asker Motoru {} için {} alanı eksik.", owner, field))
}

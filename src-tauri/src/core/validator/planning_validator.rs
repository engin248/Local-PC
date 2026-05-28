use super::push_error;
use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;

pub(crate) fn validate_planning_standard(
    planning: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let fields = planning
        .get("required_fields")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            "planning_standard.json içinde required_fields listesi bulunamadı.".to_string()
        })?;

    if fields.len() != 17 {
        push_error(
            issues,
            "PLANNING_FIELD_COUNT",
            format!(
                "Planlama standardı 17 alan bekler, bulunan: {}",
                fields.len()
            ),
        );
    }

    for field in fields {
        if field.as_str().map(|v| v.trim().is_empty()).unwrap_or(true) {
            push_error(
                issues,
                "PLANNING_FIELD_EMPTY",
                "Planlama alan listesinde boş veya geçersiz değer var.",
            );
        }
    }

    Ok(())
}

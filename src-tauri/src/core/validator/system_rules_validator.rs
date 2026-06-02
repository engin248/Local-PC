use crate::core::system_validator::SystemValidationIssue;
use serde_json::Value;
use std::collections::HashSet;

pub fn validate_system_rules(
    system_rules: &Value,
    issues: &mut Vec<SystemValidationIssue>,
) -> Result<(), String> {
    let rules_arr = match system_rules.get("system_rules").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            super::push_error(
                issues,
                "SYSTEM_RULES_INVALID_FORMAT",
                "system_rules.json içinde 'system_rules' dizisi bulunamadı veya geçersiz format.",
            );
            return Ok(());
        }
    };

    if rules_arr.is_empty() {
        super::push_error(
            issues,
            "SYSTEM_RULES_EMPTY",
            "system_rules.json içinde hiç tanımlı kural yok.",
        );
        return Ok(());
    }

    let mut ids = HashSet::new();
    for rule in rules_arr {
        let id = match rule.get("id").and_then(|v| v.as_i64()) {
            Some(i) => i,
            None => {
                super::push_error(
                    issues,
                    "SYSTEM_RULE_MISSING_ID",
                    "Bir kural için 'id' değeri bulunamadı veya geçersiz.",
                );
                continue;
            }
        };

        if !ids.insert(id) {
            super::push_error(
                issues,
                "SYSTEM_RULE_DUPLICATE_ID",
                format!("system_rules.json içinde mükerrer kural kimliği (id): {}", id),
            );
        }

        let name = match rule.get("name").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => {
                super::push_error(
                    issues,
                    "SYSTEM_RULE_MISSING_NAME",
                    format!("ID: {} kuralının 'name' alanı bulunamadı veya geçersiz.", id),
                );
                ""
            }
        };

        if name.trim().is_empty() {
            super::push_error(
                issues,
                "SYSTEM_RULE_EMPTY_NAME",
                format!("ID: {} kuralının adı boş olamaz.", id),
            );
        }

        if rule.get("description").and_then(|v| v.as_str()).is_none() {
            super::push_error(
                issues,
                "SYSTEM_RULE_MISSING_DESCRIPTION",
                format!("ID: {} kuralının 'description' alanı eksik veya geçersiz.", id),
            );
        }
    }

    Ok(())
}

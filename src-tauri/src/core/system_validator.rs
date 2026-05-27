use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

#[derive(Debug, Serialize)]
pub struct SystemValidationIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
}

pub struct SystemValidator;

impl SystemValidator {
    pub fn validate() -> Result<Vec<SystemValidationIssue>, String> {
        let mut issues = Vec::new();

        let authority = Self::read_json("authority_matrix.json")?;
        let risk = Self::read_json("risk_rules.json")?;
        let approval = Self::read_json("approval_rules.json")?;
        let rollback = Self::read_json("rollback_rules.json")?;
        let connectors = Self::read_json("system_connectors.json")?;
        let planning = Self::read_json("planning_standard.json")?;

        let action_mappings = risk
            .get("action_mappings")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "risk_rules.json içinde action_mappings bulunamadı.".to_string())?;
        let known_actions: HashSet<String> = action_mappings.keys().cloned().collect();

        let approval_actions = Self::string_array(&approval, "actions_requiring_approval")?;
        let rollback_actions = Self::string_array(&rollback, "snapshot_required_for")?;

        Self::validate_planning_standard(&planning, &mut issues)?;
        Self::validate_authority_matrix(&authority, &known_actions, &mut issues)?;
        Self::validate_risk_rules(&risk, &approval_actions, &rollback_actions, &mut issues)?;
        Self::validate_action_set("approval_rules.json", &approval_actions, &known_actions, &mut issues);
        Self::validate_action_set("rollback_rules.json", &rollback_actions, &known_actions, &mut issues);
        Self::validate_connectors(&connectors, &mut issues)?;

        Ok(issues)
    }

    pub fn validate_or_fail() -> Result<(), String> {
        let issues = Self::validate()?;
        let blockers: Vec<String> = issues
            .iter()
            .filter(|issue| issue.severity == "error")
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect();

        if blockers.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Sistem doğrulaması başarısız:\n{}",
                blockers.join("\n")
            ))
        }
    }

    fn read_json(filename: &str) -> Result<Value, String> {
        let path = DependencyAnalyzer::get_config_path(filename)?;
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("{} okunamadı: {}", filename, e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("{} JSON formatı geçersiz: {}", filename, e))
    }

    fn string_array(value: &Value, field: &str) -> Result<HashSet<String>, String> {
        let arr = value
            .get(field)
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("{} alanı liste olmalıdır.", field))?;
        let mut items = HashSet::new();
        for item in arr {
            let text = item
                .as_str()
                .ok_or_else(|| format!("{} içinde string olmayan değer var.", field))?;
            if text.trim().is_empty() {
                return Err(format!("{} içinde boş değer var.", field));
            }
            items.insert(text.to_string());
        }
        Ok(items)
    }

    fn push_error(issues: &mut Vec<SystemValidationIssue>, code: &str, message: impl Into<String>) {
        issues.push(SystemValidationIssue {
            severity: "error".to_string(),
            code: code.to_string(),
            message: message.into(),
        });
    }

    fn validate_planning_standard(
        planning: &Value,
        issues: &mut Vec<SystemValidationIssue>,
    ) -> Result<(), String> {
        let fields = planning
            .get("required_fields")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "planning_standard.json içinde required_fields listesi bulunamadı.".to_string())?;

        if fields.len() != 17 {
            Self::push_error(
                issues,
                "PLANNING_FIELD_COUNT",
                format!("Planlama standardı 17 alan bekler, bulunan: {}", fields.len()),
            );
        }

        for field in fields {
            if field.as_str().map(|v| v.trim().is_empty()).unwrap_or(true) {
                Self::push_error(issues, "PLANNING_FIELD_EMPTY", "Planlama alan listesinde boş veya geçersiz değer var.");
            }
        }

        Ok(())
    }

    fn validate_authority_matrix(
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
                Self::push_error(
                    issues,
                    "AUTHORITY_UNKNOWN_ACTION",
                    format!("Yetki matrisi risk kurallarında olmayan aksiyon içeriyor: {}", action),
                );
            }
            if deciders.as_array().map(|v| v.is_empty()).unwrap_or(true) {
                Self::push_error(
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
                Self::push_error(
                    issues,
                    "LEVEL_MAPPING_UNKNOWN_ACTION",
                    format!("Seviye eşlemesi risk kurallarında olmayan aksiyona gidiyor: {} -> {}", level, action),
                );
            }
        }

        Ok(())
    }

    fn validate_risk_rules(
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
                Self::push_error(
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
            let level = config
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !matches!(level, "low" | "medium" | "high" | "critical") {
                Self::push_error(
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
                    Self::push_error(
                        issues,
                        "RISK_ACTION_TEXT_MISSING",
                        format!("{} için {} alanı boş veya eksik.", action, field),
                    );
                }
            }
            if config.get("assets").and_then(|v| v.as_array()).is_none() {
                Self::push_error(
                    issues,
                    "RISK_ACTION_ASSETS_MISSING",
                    format!("{} için assets listesi eksik.", action),
                );
            }

            if matches!(level, "high" | "critical") && !approval_actions.contains(action) {
                Self::push_error(
                    issues,
                    "HIGH_RISK_APPROVAL_MISSING",
                    format!("Yüksek/kritik aksiyon approval_rules içinde yok: {}", action),
                );
            }
            if Self::is_write_like(action) && !rollback_actions.contains(action) {
                Self::push_error(
                    issues,
                    "WRITE_ROLLBACK_MISSING",
                    format!("Yan etkili aksiyon rollback_rules içinde yok: {}", action),
                );
            }
        }

        Ok(())
    }

    fn validate_action_set(
        source: &str,
        actions: &HashSet<String>,
        known_actions: &HashSet<String>,
        issues: &mut Vec<SystemValidationIssue>,
    ) {
        for action in actions {
            if !known_actions.contains(action) {
                Self::push_error(
                    issues,
                    "POLICY_UNKNOWN_ACTION",
                    format!("{} bilinmeyen aksiyon içeriyor: {}", source, action),
                );
            }
        }
    }

    fn validate_connectors(
        connectors: &Value,
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
            if !ids.insert(id.to_string()) {
                Self::push_error(
                    issues,
                    "CONNECTOR_DUPLICATE_ID",
                    format!("Tekrarlanan connector id: {}", id),
                );
            }
            let connector_type = connector
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Connector type alanı eksik: {}", id))?;
            let enabled = connector
                .get("enabled")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| format!("Connector enabled alanı eksik: {}", id))?;
            let dependency_level = connector
                .get("dependency_level")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("Connector dependency_level alanı eksik: {}", id))?;

            if !matches!(dependency_level, "low" | "medium" | "high" | "critical") {
                Self::push_error(
                    issues,
                    "CONNECTOR_LEVEL_INVALID",
                    format!("Connector dependency_level geçersiz: {} -> {}", id, dependency_level),
                );
            }
            if matches!(connector_type, "folder" | "sqlite")
                && connector.get("path").and_then(|v| v.as_str()).map(|v| v.trim().is_empty()).unwrap_or(true)
            {
                Self::push_error(
                    issues,
                    "CONNECTOR_PATH_MISSING",
                    format!("{} connector path alanı olmadan kullanılamaz.", id),
                );
            }
            if connector_type == "api" && enabled {
                let base_url_empty = connector
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true);
                if base_url_empty {
                    Self::push_error(
                        issues,
                        "CONNECTOR_API_BASE_URL_MISSING",
                        format!("Etkin API connector base_url olmadan çalışamaz: {}", id),
                    );
                }
            }
        }

        Ok(())
    }

    fn is_write_like(action: &str) -> bool {
        action.contains("write")
            || action.contains("delete")
            || action == "terminal_command"
            || action == "live_system_update"
    }
}

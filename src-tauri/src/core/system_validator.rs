use crate::core::dependency_analyzer::DependencyAnalyzer;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

#[path = "validator/mod.rs"]
mod validator;

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
        let ai_providers = Self::read_json("ai_providers.json")?;
        let planning = Self::read_json("planning_standard.json")?;
        let decision_principles = Self::read_json("decision_principles.json")?;

        let action_mappings = risk
            .get("action_mappings")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "risk_rules.json içinde action_mappings bulunamadı.".to_string())?;
        let known_actions: HashSet<String> = action_mappings.keys().cloned().collect();

        let approval_actions = Self::string_array(&approval, "actions_requiring_approval")?;
        let rollback_actions = Self::string_array(&rollback, "snapshot_required_for")?;

        validator::planning_validator::validate_planning_standard(&planning, &mut issues)?;
        validator::authority_validator::validate_authority_matrix(
            &authority,
            &known_actions,
            &mut issues,
        )?;
        validator::risk_validator::validate_risk_rules(
            &risk,
            &approval_actions,
            &rollback_actions,
            &mut issues,
        )?;
        validator::approval_validator::validate_approval_action_set(
            &approval_actions,
            &known_actions,
            &mut issues,
        );
        validator::rollback_validator::validate_rollback_action_set(
            &rollback_actions,
            &known_actions,
            &mut issues,
        );
        validator::providers_validator::validate_ai_providers(&ai_providers, &mut issues)?;
        validator::connectors_validator::validate_connectors(
            &connectors,
            &known_actions,
            &mut issues,
        )?;
        Self::validate_decision_principles(&decision_principles, &mut issues)?;

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
        let data =
            fs::read_to_string(&path).map_err(|e| format!("{} okunamadı: {}", filename, e))?;
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

    fn validate_decision_principles(
        principles: &Value,
        issues: &mut Vec<SystemValidationIssue>,
    ) -> Result<(), String> {
        for field in [
            "correct_approach_criteria",
            "best_option_criteria",
            "required_reason_fields",
            "human_accepted_principles",
            "alternative_scoring_weights",
            "high_risk_requirements",
            "critical_risk_requirements",
            "phase_model",
        ] {
            if principles.get(field).is_none() {
                validator::push_error(
                    issues,
                    "DECISION_PRINCIPLES_MISSING_FIELD",
                    format!(
                        "decision_principles.json içinde zorunlu alan eksik: {}",
                        field
                    ),
                );
            }
        }

        let required_reasons = principles
            .get("required_reason_fields")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "required_reason_fields liste olmalıdır.".to_string())?;
        for reason in [
            "accepted_correct_approach_reason",
            "selected_best_option_reason",
        ] {
            if !required_reasons.iter().any(|v| v.as_str() == Some(reason)) {
                validator::push_error(
                    issues,
                    "DECISION_PRINCIPLES_REASON_MISSING",
                    format!("Zorunlu gerekçe alanı eksik: {}", reason),
                );
            }
        }

        let phase_model = principles
            .get("phase_model")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "phase_model nesne olmalıdır.".to_string())?;
        for phase in [
            "analysis",
            "plan_ready",
            "awaiting_approval",
            "execution",
            "monitoring",
            "verification",
            "completed",
            "failed",
            "rolled_back",
        ] {
            if phase_model
                .get(phase)
                .and_then(|v| v.as_str())
                .map(|v| v.trim().is_empty())
                .unwrap_or(true)
            {
                validator::push_error(
                    issues,
                    "PHASE_MODEL_INCOMPLETE",
                    format!("Phase eşleştirmesi eksik veya boş: {}", phase),
                );
            }
        }

        Ok(())
    }
}
#[test]
fn print_validation_issues() {
    let issues = crate::core::system_validator::SystemValidator::validate();
    println!("ISSUES: {:#?}", issues);
}
#[cfg(test)]
mod system_tests {
    use super::*;
    #[test]
    fn run_validator() {
        let issues = SystemValidator::validate();
        println!("ISSUES_DUMP: {:#?}", issues);
        assert!(issues.is_ok());
    }
}

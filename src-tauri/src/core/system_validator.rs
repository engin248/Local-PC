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

        Self::validate_planning_standard(&planning, &mut issues)?;
        Self::validate_authority_matrix(&authority, &known_actions, &mut issues)?;
        Self::validate_risk_rules(&risk, &approval_actions, &rollback_actions, &mut issues)?;
        Self::validate_action_set(
            "approval_rules.json",
            &approval_actions,
            &known_actions,
            &mut issues,
        );
        Self::validate_action_set(
            "rollback_rules.json",
            &rollback_actions,
            &known_actions,
            &mut issues,
        );
        Self::validate_ai_providers(&ai_providers, &mut issues)?;
        Self::validate_connectors(&connectors, &known_actions, &mut issues)?;
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
            .ok_or_else(|| {
                "planning_standard.json içinde required_fields listesi bulunamadı.".to_string()
            })?;

        if fields.len() != 17 {
            Self::push_error(
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
                Self::push_error(
                    issues,
                    "PLANNING_FIELD_EMPTY",
                    "Planlama alan listesinde boş veya geçersiz değer var.",
                );
            }
        }

        Ok(())
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
                    issues,
                    "PHASE_MODEL_INCOMPLETE",
                    format!("Phase eşleştirmesi eksik veya boş: {}", phase),
                );
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
                    format!(
                        "Yetki matrisi risk kurallarında olmayan aksiyon içeriyor: {}",
                        action
                    ),
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
                    format!(
                        "Seviye eşlemesi risk kurallarında olmayan aksiyona gidiyor: {} -> {}",
                        level, action
                    ),
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
            let level = config.get("level").and_then(|v| v.as_str()).unwrap_or("");
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
                    format!(
                        "Yüksek/kritik aksiyon approval_rules içinde yok: {}",
                        action
                    ),
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

    fn validate_ai_providers(
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
            Self::validate_no_production_placeholder(
                "AI_PROVIDER_PRODUCTION_PLACEHOLDER",
                id,
                provider,
                issues,
            );
            if !ids.insert(id.to_string()) {
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
                    issues,
                    "AI_PROVIDER_ALLOWED_TASKS_MISSING",
                    format!("AI provider allowed_task_types boş veya eksik: {}", id),
                );
            }
            if provider.get("sensitive_data_policy").is_none() {
                Self::push_error(
                    issues,
                    "AI_PROVIDER_SENSITIVE_POLICY_MISSING",
                    format!("AI provider sensitive_data_policy eksik: {}", id),
                );
            }
            if provider.get("max_payload_policy").is_none() {
                Self::push_error(
                    issues,
                    "AI_PROVIDER_PAYLOAD_POLICY_MISSING",
                    format!("AI provider max_payload_policy eksik: {}", id),
                );
            }
        }

        Ok(())
    }

    fn validate_connectors(
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
            Self::validate_no_production_placeholder(
                "CONNECTOR_PRODUCTION_PLACEHOLDER",
                id,
                connector,
                issues,
            );
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
            if !matches!(
                connector_type,
                "folder" | "file" | "sqlite" | "api" | "live_api" | "terminal" | "custom_connector"
            ) {
                Self::push_error(
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
                Self::push_error(
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
                Self::push_error(
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
                    Self::push_error(
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
                    Self::push_error(
                        issues,
                        "CONNECTOR_ACTION_LIST_MISSING",
                        format!("{} connector {} alanı liste olmalıdır.", id, field),
                    );
                }
            }

            if connector.get("read_only_default").and_then(|v| v.as_bool()) != Some(true) {
                Self::push_error(
                    issues,
                    "CONNECTOR_READ_ONLY_DEFAULT_MISSING",
                    format!("{} connector read_only_default=true olmalıdır.", id),
                );
            }

            let permissions = Self::optional_string_set(connector, "permissions");
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
                    Self::push_error(
                        issues,
                        "CONNECTOR_PERMISSION_INVALID",
                        format!("{} connector izni geçersiz: {}", id, permission),
                    );
                }
            }

            let allowed_actions = Self::optional_string_set(connector, "allowed_actions");
            let approval_required_actions =
                Self::optional_string_set(connector, "approval_required_actions");
            let rollback_required_actions =
                Self::optional_string_set(connector, "rollback_required_actions");
            let test_required_actions =
                Self::optional_string_set(connector, "test_required_actions");

            for action in allowed_actions
                .iter()
                .chain(approval_required_actions.iter())
                .chain(rollback_required_actions.iter())
                .chain(test_required_actions.iter())
            {
                if !known_actions.contains(action) {
                    Self::push_error(
                        issues,
                        "CONNECTOR_UNKNOWN_ACTION",
                        format!("{} connector bilinmeyen aksiyon içeriyor: {}", id, action),
                    );
                }
            }

            let risky_actions: HashSet<String> = allowed_actions
                .iter()
                .chain(approval_required_actions.iter())
                .filter(|action| Self::is_write_like(action))
                .cloned()
                .collect();
            for action in risky_actions {
                if !approval_required_actions.contains(&action) {
                    Self::push_error(
                        issues,
                        "CONNECTOR_APPROVAL_ACTION_MISSING",
                        format!("{} riskli aksiyon approval listesinde yok: {}", id, action),
                    );
                }
                if !rollback_required_actions.contains(&action) {
                    Self::push_error(
                        issues,
                        "CONNECTOR_ROLLBACK_ACTION_MISSING",
                        format!("{} riskli aksiyon rollback listesinde yok: {}", id, action),
                    );
                }
                if !test_required_actions.contains(&action) {
                    Self::push_error(
                        issues,
                        "CONNECTOR_TEST_ACTION_MISSING",
                        format!("{} riskli aksiyon test listesinde yok: {}", id, action),
                    );
                }
            }
        }

        Ok(())
    }

    fn validate_no_production_placeholder(
        code: &str,
        id: &str,
        item: &Value,
        issues: &mut Vec<SystemValidationIssue>,
    ) {
        if item.get("placeholder").and_then(|v| v.as_bool()) == Some(true) {
            Self::push_error(
                issues,
                code,
                format!("Production config placeholder kaydı içeremez: {}", id),
            );
        }

        if Self::contains_forbidden_production_text(item) {
            Self::push_error(
                issues,
                code,
                format!(
                    "Production config şablon/example/demo/mock metni içeremez: {}",
                    id
                ),
            );
        }
    }

    fn contains_forbidden_production_text(value: &Value) -> bool {
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
            Value::Array(items) => items.iter().any(Self::contains_forbidden_production_text),
            Value::Object(map) => map.values().any(Self::contains_forbidden_production_text),
            _ => false,
        }
    }

    fn optional_string_set(value: &Value, field: &str) -> HashSet<String> {
        match value.get(field).and_then(|v| v.as_array()) {
            Some(items) => items
                .iter()
                .filter_map(|item| item.as_str().map(|text| text.to_string()))
                .collect(),
            None => HashSet::new(),
        }
    }

    fn is_write_like(action: &str) -> bool {
        action.contains("write")
            || action.contains("delete")
            || action == "terminal_command"
            || action == "live_system_update"
    }
}

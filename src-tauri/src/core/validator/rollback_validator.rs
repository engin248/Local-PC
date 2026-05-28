use super::validate_action_set;
use crate::core::system_validator::SystemValidationIssue;
use std::collections::HashSet;

pub(crate) fn validate_rollback_action_set(
    rollback_actions: &HashSet<String>,
    known_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) {
    validate_action_set(
        "rollback_rules.json",
        rollback_actions,
        known_actions,
        issues,
    );
}

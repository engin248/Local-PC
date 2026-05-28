use super::validate_action_set;
use crate::core::system_validator::SystemValidationIssue;
use std::collections::HashSet;

pub(crate) fn validate_approval_action_set(
    approval_actions: &HashSet<String>,
    known_actions: &HashSet<String>,
    issues: &mut Vec<SystemValidationIssue>,
) {
    validate_action_set(
        "approval_rules.json",
        approval_actions,
        known_actions,
        issues,
    );
}

use crate::storage::db::Database;
use rusqlite::params;
use uuid::Uuid;

pub struct ApprovalManager;

impl ApprovalManager {
    pub fn request_approval(
        task_id: &str,
        node_id: Option<&str>,
        action: &str,
        risk_level: &str,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO approvals (id, task_id, decision_node_id, action, risk_level, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
            params![id, task_id, node_id, action, risk_level],
        )
        .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn ensure_pending_approval_requests(
        task_id: &str,
        node_id: Option<&str>,
        action: &str,
        risk_level: &str,
        required_count: i32,
    ) -> Result<Vec<String>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let existing_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM approvals
                 WHERE task_id = ?1
                 AND ((decision_node_id IS NULL AND ?2 IS NULL) OR decision_node_id = ?2)
                 AND action = ?3
                 AND risk_level = ?4
                 AND status IN ('pending', 'approved')",
                params![task_id, node_id, action, risk_level],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let mut created_ids = Vec::new();
        for _ in existing_count..required_count {
            created_ids.push(Self::request_approval(
                task_id, node_id, action, risk_level,
            )?);
        }

        Ok(created_ids)
    }

    pub fn check_approval_status(approval_id: &str) -> Result<bool, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let (status, approver_id, approver_role, risk_level): (
            String,
            Option<String>,
            Option<String>,
            String,
        ) = conn.query_row(
            "SELECT status, approver_id, approver_role, risk_level FROM approvals WHERE id = ?1",
            params![approval_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).map_err(|e| e.to_string())?;

        if status != "approved" {
            return Ok(false);
        }

        let approver_id =
            approver_id.ok_or_else(|| "HATA: Onay kaydında approver_id yok.".to_string())?;
        let approver_role =
            approver_role.ok_or_else(|| "HATA: Onay kaydında approver_role yok.".to_string())?;

        Ok(!approver_id.trim().is_empty()
            && Self::role_is_authorized_for_risk(&approver_role, &risk_level))
    }

    pub fn role_is_authorized_for_risk(role: &str, risk_level: &str) -> bool {
        match risk_level {
            "high" | "critical" => matches!(role, "admin" | "owner" | "security_officer"),
            "medium" => matches!(role, "admin" | "owner" | "security_officer" | "operator"),
            "low" => matches!(
                role,
                "admin" | "owner" | "security_officer" | "operator" | "user"
            ),
            _ => false,
        }
    }

    pub fn approve_request(
        approval_id: &str,
        user_note: &str,
        approver_id: &str,
        approver_role: &str,
        approval_source: &str,
    ) -> Result<(), String> {
        if approver_id.trim().is_empty() {
            return Err("HATA: Onay veren kullanıcı kimliği boş olamaz.".to_string());
        }

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        let risk_level: String = conn
            .query_row(
                "SELECT risk_level FROM approvals WHERE id = ?1",
                params![approval_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if !Self::role_is_authorized_for_risk(approver_role, &risk_level) {
            return Err(format!(
                "HATA: {} riski için onay veren rol yetkili değil: {}",
                risk_level, approver_role
            ));
        }

        conn.execute(
            "UPDATE approvals
             SET status = 'approved',
                 approved_at = CURRENT_TIMESTAMP,
                 user_note = ?1,
                 approver_id = ?2,
                 approver_role = ?3,
                 approval_source = ?4
             WHERE id = ?5",
            params![
                user_note,
                approver_id,
                approver_role,
                approval_source,
                approval_id
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

pub fn submit_approval(
    approval_id: &str,
    approve: bool,
    user_note: Option<&str>,
    approver_id: Option<&str>,
    approver_role: Option<&str>,
    approval_source: Option<&str>,
) -> Result<(), String> {
    let note = user_note.unwrap_or("");
    let approver_id = approver_id
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "HATA: Onay için gerçek kullanıcı kimliği zorunludur.".to_string())?;
    let approver_role = approver_role
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "HATA: Onay için gerçek kullanıcı rolü zorunludur.".to_string())?;
    let approval_source = approval_source
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "HATA: Onay kaynağı zorunludur.".to_string())?;

    if approve {
        return ApprovalManager::approve_request(
            approval_id,
            note,
            approver_id,
            approver_role,
            approval_source,
        );
    }

    let status = if approve { "approved" } else { "rejected" };
    let db = Database::new();
    let conn = db.get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE approvals
         SET status = ?1,
             approved_at = CURRENT_TIMESTAMP,
             user_note = ?2,
             approver_id = ?3,
             approver_role = ?4,
             approval_source = ?5
         WHERE id = ?6",
        params![
            status,
            note,
            approver_id,
            approver_role,
            approval_source,
            approval_id
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{submit_approval, ApprovalManager};
    use crate::storage::db::Database;
    use rusqlite::params;

    #[test]
    fn ai_statement_cannot_act_as_approval() {
        let task_id = "test_ai_statement_cannot_approve";
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Test', 'Test', 'pending', 'planning_complete', 'not_started', 'high', 'pending_approval')",
            params![task_id],
        )
        .unwrap();
        let approval_id =
            ApprovalManager::request_approval(task_id, None, "write_file", "high").unwrap();
        let result = submit_approval(
            &approval_id,
            true,
            Some("AI provider beyanı onay yerine geçmesin."),
            Some("chatgpt"),
            Some("ai_provider"),
            Some("ai_statement"),
        );
        assert!(result.is_err());
    }
}

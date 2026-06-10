use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub trait SystemConnector {
    fn execute_read(&self, target: &str) -> Result<String, String>;
    fn execute_write(&self, target: &str, data: &str) -> Result<(), String>;
    fn get_name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConnectorConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub connector_type: String,
    pub path: Option<String>,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub permissions: Vec<String>,
    pub enabled: bool,
    pub dependency_level: String,
    pub live_system: bool,
    pub network_required: bool,
    pub allowed_actions: Vec<String>,
    pub approval_required_actions: Vec<String>,
    pub rollback_required_actions: Vec<String>,
    pub test_required_actions: Vec<String>,
    pub read_only_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConnectorHealth {
    pub id: String,
    pub name: String,
    pub connector_type: String,
    pub source_kind: String,
    pub target: Option<String>,
    pub source_path: Option<String>,
    pub endpoint: Option<String>,
    pub permissions: Vec<String>,
    pub enabled: bool,
    pub read_only: bool,
    pub dependency_level: String,
    pub live_system: bool,
    pub network_required: bool,
    pub allowed_actions: Vec<String>,
    pub approval_required_actions: Vec<String>,
    pub rollback_required_actions: Vec<String>,
    pub test_required_actions: Vec<String>,
    pub status: String,
    pub health: String,
    pub last_error: Option<String>,
    pub last_checked_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteApprovalContext {
    pub task_id: String,
    pub decision_node_id: String,
    pub action: String,
    pub risk_level: String,
}

#[derive(Debug, Deserialize)]
struct WriteRequestEnvelope {
    approval_context: WriteApprovalContext,
    payload: Option<String>,
}

impl WriteApprovalContext {
    fn validate(&self, expected_action: &str) -> Result<(), String> {
        if self.task_id.trim().is_empty() {
            return Err("HATA: Yazma isteği task_id olmadan çalıştırılamaz.".to_string());
        }
        if self.decision_node_id.trim().is_empty() {
            return Err("HATA: Yazma isteği decision_node_id olmadan çalıştırılamaz.".to_string());
        }
        if self.action != expected_action {
            return Err(format!(
                "HATA: Yazma isteği aksiyon bağlamı eşleşmiyor. Beklenen: {}, gelen: {}",
                expected_action, self.action
            ));
        }
        if !matches!(self.risk_level.as_str(), "high" | "critical") {
            return Err(format!(
                "HATA: Yazma isteği yüksek/kritik risk bağlamı olmadan çalıştırılamaz. Gelen risk: {}",
                self.risk_level
            ));
        }

        Ok(())
    }
}

pub fn decode_write_request(
    expected_action: &str,
    data: &str,
) -> Result<(WriteApprovalContext, String), String> {
    let envelope: WriteRequestEnvelope = serde_json::from_str(data).map_err(|e| {
        format!(
            "HATA: Yazma isteği geçerli approval_context JSON zarfı içermelidir: {}",
            e
        )
    })?;

    envelope.approval_context.validate(expected_action)?;

    Ok((
        envelope.approval_context,
        envelope
            .payload
            .filter(|payload| !payload.trim().is_empty())
            .ok_or_else(|| "HATA: Yazma isteği boş payload ile çalıştırılamaz.".to_string())?,
    ))
}

pub fn require_authorized_write(context: &WriteApprovalContext) -> Result<(), String> {
    context.validate(&context.action)?;

    let db = Database::new();
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(DISTINCT approver_id) FROM approvals
             WHERE task_id = ?1
             AND decision_node_id = ?2
             AND action = ?3
             AND risk_level = ?4
             AND status = 'approved'
             AND approved_at IS NOT NULL
             AND approver_id IS NOT NULL
             AND TRIM(approver_id) != ''
             AND approver_role IN ('admin', 'owner', 'security_officer')
             AND approval_source IN ('ui', 'policy', 'database')",
            params![
                context.task_id,
                context.decision_node_id,
                context.action,
                context.risk_level
            ],
            |row| row.get(0),
        )
        .map_err(|e| format!("HATA: Yazma onayı sorgulanamadı: {}", e))?;

    let required_count = if matches!(context.risk_level.as_str(), "high" | "critical") {
        2
    } else {
        1
    };

    if count < required_count {
        return Err(format!(
            "HATA: {} işlemi yetkilendirilmemiştir. Görev/düğüm/risk bağlamına bağlı en az {} ayrı yetkili onay gerekir; bulunan: {}.",
            context.action, required_count, count
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{require_authorized_write, WriteApprovalContext};
    use crate::storage::db::Database;
    use rusqlite::params;

    fn reset(task_id: &str) {
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        let _ = conn.execute("DELETE FROM approvals WHERE task_id = ?1", params![task_id]);
        let _ = conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id]);
        conn.execute(
            "INSERT INTO tasks (id, title, user_request, status, planning_status, execution_status, risk_level, approval_status)
             VALUES (?1, 'Connector Write Test', 'Connector write authorization test', 'pending', 'planning_complete', 'not_started', 'high', 'pending_approval')",
            params![task_id],
        )
        .unwrap();
    }

    fn insert_approval(task_id: &str, approval_id: &str, approver_id: &str) {
        let db = Database::new();
        let conn = db.get_connection().unwrap();
        conn.execute(
            "INSERT INTO approvals (
                id, task_id, decision_node_id, approver_id, approver_role, approval_source,
                action, risk_level, approved_at, status
             ) VALUES (?1, ?2, 'node_a', ?3, 'admin', 'database', 'write_file', 'high', CURRENT_TIMESTAMP, 'approved')",
            params![approval_id, task_id, approver_id],
        )
        .unwrap();
    }

    #[test]
    fn high_risk_connector_write_requires_two_distinct_authorized_approvals() {
        let task_id = "test_connector_double_approval";
        reset(task_id);
        insert_approval(task_id, "approval_one", "admin_one");

        let context = WriteApprovalContext {
            task_id: task_id.to_string(),
            decision_node_id: "node_a".to_string(),
            action: "write_file".to_string(),
            risk_level: "high".to_string(),
        };

        assert!(require_authorized_write(&context).is_err());

        insert_approval(task_id, "approval_two", "admin_two");
        assert!(require_authorized_write(&context).is_ok());
    }
}

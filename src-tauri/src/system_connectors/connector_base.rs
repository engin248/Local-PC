use crate::storage::db::Database;
use rusqlite::params;
use serde::Deserialize;

pub trait SystemConnector {
    fn execute_read(&self, target: &str) -> Result<String, String>;
    fn execute_write(&self, target: &str, data: &str) -> Result<(), String>;
    fn get_name(&self) -> &str;
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

    Ok((envelope.approval_context, envelope.payload.unwrap_or_default()))
}

pub fn require_authorized_write(context: &WriteApprovalContext) -> Result<(), String> {
    context.validate(&context.action)?;

    let db = Database::new();
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM approvals
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

    if count == 0 {
        return Err(format!(
            "HATA: {} işlemi yetkilendirilmemiştir. Görev/düğüm/risk bağlamına bağlı geçerli, tarihli ve yetkili onay kaydı bulunamadı.",
            context.action
        ));
    }

    Ok(())
}

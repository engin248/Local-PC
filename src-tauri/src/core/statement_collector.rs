use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Statement {
    pub id: String,
    pub decision_node_id: String,
    pub source_type: String,
    pub source_name: String,
    pub content: String,
    pub evidence_ref: Option<String>,
}

pub struct StatementCollector;

impl StatementCollector {
    pub fn collect_statement(
        node_id: &str,
        src_type: &str,
        src_name: &str,
        content: &str,
        evidence_ref: Option<&str>,
    ) -> Result<Statement, String> {
        // Dynamic validation based on source type
        let validated_content = match src_type {
            "ai_provider" | "ai_provider_response" => {
                if content.trim().is_empty() {
                    return Err("HATA: Yapay zeka beyan içeriği boş olamaz!".to_string());
                }
                if evidence_ref.map(|s| s.trim().is_empty()).unwrap_or(true) {
                    return Err(
                        "HATA: Yapay zeka beyanı gerçek evidence_ref olmadan kabul edilemez."
                            .to_string(),
                    );
                }
                format!("[AI-VERIFIED] {}", content)
            }
            "system_connector" | "system_connector_output" => {
                let connectors_path =
                    crate::core::dependency_analyzer::DependencyAnalyzer::get_config_path(
                        "system_connectors.json",
                    )?;
                let mut is_valid = false;
                let data = std::fs::read_to_string(&connectors_path)
                    .map_err(|e| format!("Sistem connector config okunamadı: {}", e))?;
                let val: serde_json::Value = serde_json::from_str(&data)
                    .map_err(|e| format!("Sistem connector config JSON formatı geçersiz: {}", e))?;
                let arr = val.as_array().ok_or_else(|| {
                    "Sistem connector config liste formatında olmalıdır.".to_string()
                })?;
                for item in arr {
                    let id = item.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                        "Sistem connector config içinde id alanı eksik.".to_string()
                    })?;
                    let item_type = item.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
                        "Sistem connector config içinde type alanı eksik.".to_string()
                    })?;
                    let enabled =
                        item.get("enabled")
                            .and_then(|v| v.as_bool())
                            .ok_or_else(|| {
                                format!(
                                    "Sistem connector config içinde enabled alanı eksik: {}",
                                    id
                                )
                            })?;
                    if enabled
                        && (id == src_name
                            || item_type == src_name
                            || format!("{}_connector", item_type) == src_name)
                    {
                        is_valid = true;
                        break;
                    }
                }

                if !is_valid {
                    return Err(format!(
                        "HATA: Geçersiz veya devre dışı sistem konnektör kaynağı: {}",
                        src_name
                    ));
                }
                format!("[SYS-VERIFIED] Kaynak: {}, Veri: {}", src_name, content)
            }
            "user_instruction" => {
                if content.trim().is_empty() {
                    return Err("HATA: Kullanıcı talimatı beyan içeriği boş olamaz!".to_string());
                }
                format!("[USER-VERIFIED] {}", content)
            }
            "audit_log" | "test_result" | "sqlite_read" | "file_read" => {
                if evidence_ref.map(|s| s.trim().is_empty()).unwrap_or(true) {
                    return Err(format!(
                        "HATA: {} beyanı gerçek evidence_ref olmadan kabul edilemez.",
                        src_type
                    ));
                }
                format!("[VERIFIED] {}", content)
            }
            _ => {
                return Err(format!(
                    "HATA: Desteklenmeyen beyan kaynak türü: {}",
                    src_type
                ))
            }
        };

        let stmt = Statement {
            id: Uuid::new_v4().to_string(),
            decision_node_id: node_id.to_string(),
            source_type: src_type.to_string(),
            source_name: src_name.to_string(),
            content: validated_content,
            evidence_ref: evidence_ref.map(|s| s.to_string()),
        };

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO statements (id, decision_node_id, source_type, source_name, content, evidence_ref)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                stmt.id,
                stmt.decision_node_id,
                stmt.source_type,
                stmt.source_name,
                stmt.content,
                stmt.evidence_ref
            ],
        ).map_err(|e| e.to_string())?;

        Ok(stmt)
    }

    pub fn get_statements(node_id: &str) -> Result<Vec<Statement>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, decision_node_id, source_type, source_name, content, evidence_ref FROM statements WHERE decision_node_id = ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![node_id], |row| {
                Ok(Statement {
                    id: row.get(0)?,
                    decision_node_id: row.get(1)?,
                    source_type: row.get(2)?,
                    source_name: row.get(3)?,
                    content: row.get(4)?,
                    evidence_ref: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut stmts = Vec::new();
        for row in rows {
            stmts.push(row.map_err(|e| e.to_string())?);
        }

        Ok(stmts)
    }
}

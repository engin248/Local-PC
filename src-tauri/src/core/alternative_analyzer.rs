use serde::{Serialize, Deserialize};
use rusqlite::params;
use uuid::Uuid;
use crate::storage::db::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alternative {
    pub id: String,
    pub decision_node_id: String,
    pub title: String,
    pub description: String,
    pub pros_json: String,
    pub cons_json: String,
    pub accuracy_score: i32,
    pub safety_score: i32,
    pub dependency_score: i32,
    pub rollback_score: i32,
    pub maintainability_score: i32,
    pub cost_score: i32,
    pub time_score: i32,
    pub user_control_score: i32,
    pub live_impact_score: i32,
    pub data_loss_risk_score: i32,
    pub selected: i32,
    pub reason: Option<String>,
}

pub struct AlternativeAnalyzer;

impl AlternativeAnalyzer {
    pub fn generate_alternatives(node_id: &str) -> Result<Vec<Alternative>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // Retrieve user request dynamically via node_id
        let user_request: String = conn.query_row(
            "SELECT t.user_request FROM tasks t 
             JOIN decision_nodes n ON t.id = n.task_id 
             WHERE n.id = ?1",
            params![node_id],
            |row| row.get(0),
        )
        .map_err(|e| {
            format!(
                "Alternatif analizi için görev metni bulunamadı veya okunamadı (node_id={}): {}",
                node_id, e
            )
        })?;

        let keyword = if user_request.to_lowercase().contains("dosya") || user_request.to_lowercase().contains("file") {
            "Dosya Modifikasyonu"
        } else if user_request.to_lowercase().contains("veritabanı") || user_request.to_lowercase().contains("db") {
            "Veritabanı İşlemi"
        } else {
            "Sistem Operasyonu"
        };

        let alts = vec![
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Yedek Alıp Kullanıcı Onayıyla {} Yapma", keyword),
                description: format!("Değişiklik öncesi kapsamlı snapshot alınır, kullanıcı onayına sunulur ve {} gerçekleştirilir.", keyword),
                pros_json: "[\"Geri alınabilir\", \"Kullanıcı kontrolü yüksek\", \"Düşük risk\"]".to_string(),
                cons_json: "[\"Zaman alıcı\", \"Ekran onayı bekler\"]".to_string(),
                accuracy_score: 9,
                safety_score: 10,
                dependency_score: 10,
                rollback_score: 10,
                maintainability_score: 9,
                cost_score: 10,
                time_score: 7,
                user_control_score: 10,
                live_impact_score: 2,
                data_loss_risk_score: 1,
                selected: 1,
                reason: Some(format!("Kullanıcı talebine ({}) göre en güvenli, geri alınabilir ve kontrollü yöntem.", keyword)),
            },
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Doğrudan Otonom {} Uygulama", keyword),
                description: format!("Yapay zeka kararına güvenilerek {} doğrudan ve onaysız olarak yürütülür.", keyword),
                pros_json: "[\"Çok hızlı\", \"Sıfır ek adım\"]".to_string(),
                cons_json: "[\"Yüksek risk\", \"Geri alınamaz\"]".to_string(),
                accuracy_score: 5,
                safety_score: 2,
                dependency_score: 8,
                rollback_score: 1,
                maintainability_score: 5,
                cost_score: 10,
                time_score: 10,
                user_control_score: 1,
                live_impact_score: 9,
                data_loss_risk_score: 9,
                selected: 0,
                reason: None,
            },
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Sadece Öneri ve {} Raporu Hazırlama", keyword),
                description: format!("Sistem üzerinde doğrudan işlem yapılmaz, sadece yapılacak {} için rapor hazırlanır.", keyword),
                pros_json: "[\"Sıfır operasyonel risk\", \"Kolay denetim\"]".to_string(),
                cons_json: "[\"İşlem tamamlanmaz\", \"Manuel müdahale gerektirir\"]".to_string(),
                accuracy_score: 10,
                safety_score: 10,
                dependency_score: 10,
                rollback_score: 10,
                maintainability_score: 8,
                cost_score: 5,
                time_score: 3,
                user_control_score: 10,
                live_impact_score: 0,
                data_loss_risk_score: 0,
                selected: 0,
                reason: None,
            }
        ];

        for alt in &alts {
            conn.execute(
                "INSERT INTO alternatives (id, decision_node_id, title, description, pros_json, cons_json, accuracy_score, safety_score, dependency_score, rollback_score, maintainability_score, cost_score, time_score, user_control_score, live_impact_score, data_loss_risk_score, selected, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    alt.id,
                    alt.decision_node_id,
                    alt.title,
                    alt.description,
                    alt.pros_json,
                    alt.cons_json,
                    alt.accuracy_score,
                    alt.safety_score,
                    alt.dependency_score,
                    alt.rollback_score,
                    alt.maintainability_score,
                    alt.cost_score,
                    alt.time_score,
                    alt.user_control_score,
                    alt.live_impact_score,
                    alt.data_loss_risk_score,
                    alt.selected,
                    alt.reason
                ],
            ).map_err(|e| e.to_string())?;
        }

        Ok(alts)
    }

    pub fn get_alternatives(node_id: &str) -> Result<Vec<Alternative>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, decision_node_id, title, description, pros_json, cons_json, accuracy_score, safety_score, dependency_score, rollback_score, maintainability_score, cost_score, time_score, user_control_score, live_impact_score, data_loss_risk_score, selected, reason FROM alternatives WHERE decision_node_id = ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![node_id], |row| {
            Ok(Alternative {
                id: row.get(0)?,
                decision_node_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                pros_json: row.get(4)?,
                cons_json: row.get(5)?,
                accuracy_score: row.get(6)?,
                safety_score: row.get(7)?,
                dependency_score: row.get(8)?,
                rollback_score: row.get(9)?,
                maintainability_score: row.get(10)?,
                cost_score: row.get(11)?,
                time_score: row.get(12)?,
                user_control_score: row.get(13)?,
                live_impact_score: row.get(14)?,
                data_loss_risk_score: row.get(15)?,
                selected: row.get(16)?,
                reason: row.get(17)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut alts = Vec::new();
        for row in rows {
            alts.push(row.map_err(|e| e.to_string())?);
        }

        Ok(alts)
    }
}

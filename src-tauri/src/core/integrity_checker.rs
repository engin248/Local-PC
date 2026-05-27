use crate::storage::db::Database;
use rusqlite::params;

pub struct IntegrityChecker;

impl IntegrityChecker {
    pub fn check_integrity(task_id: &str) -> Result<(), String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // 1. Dynamic Check for conflicting decisions or statements (Dynamic Integrity Check)
        let conflict_exists: bool = conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM statements s1 
                JOIN statements s2 ON s1.decision_node_id = s2.decision_node_id
                WHERE s1.content LIKE '%HATA%' AND s2.content LIKE '%BAŞARI%'
             )",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("Bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if conflict_exists {
            return Err("Bütünlük Hatası: Beyanlar arasında çelişkili kararlar veya çakışan bildirimler tespit edildi!".to_string());
        }

        // 2. Verify all decision nodes are approved/passed
        let unpassed_nodes_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM decision_nodes WHERE task_id = ?1 AND status != 'passed' AND status != 'completed'",
            params![task_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Karar düğümü bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if unpassed_nodes_count > 0 {
            return Err(format!("Bütünlük Hatası: Karar düğümleri tamamlanmamış durumda! Eksik karar düğümü sayısı: {}", unpassed_nodes_count));
        }

        // 3. Verify all tests passed
        let failed_tests_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM tests WHERE task_id = ?1 AND status = 'failed'",
            params![task_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Test bütünlük sorgusu çalıştırılamadı: {}", e))?;

        if failed_tests_count > 0 {
            return Err("Bütünlük Hatası: Başarısız testler mevcut olduğundan işlem tamamlanamaz!".to_string());
        }

        Ok(())
    }
}

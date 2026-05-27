use serde::{Serialize, Deserialize};
use rusqlite::params;
use uuid::Uuid;
use crate::storage::db::Database;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskBreakdown {
    pub id: String,
    pub task_id: String,
    pub parent_id: Option<String>,
    pub level: i32,
    pub topic: String,
    pub subtopic: String,
    pub criterion: String,
    pub subcriterion: String,
    pub description: Option<String>,
    pub risk_pre_label: Option<String>,
    pub probable_connector: Option<String>,
    pub decision_node_required: Option<String>,
}

pub struct TaskDecomposer;

impl TaskDecomposer {
    pub fn decompose_task(task_id: &str, user_request: &str) -> Result<Vec<TaskBreakdown>, String> {
        let mut breakdowns = Vec::new();
        
        // Dynamically split user_request into sentences or lines
        let lines: Vec<&str> = user_request.split(&['.', '\n', ';'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // Fallback to real local-control steps if user_request is too short.
        let lines = if lines.is_empty() {
            vec![
                "Kullanıcı talimatını ve hedef etki alanını doğrula.",
                "Yetki matrisi atamasını doğrula.",
                "Alternatifleri risk ve geri alınabilirlik kriterleriyle derecelendir.",
                "Gerçek hedef için snapshot ve rollback yedeği oluştur.",
                "Test ve bütünlük kontrollerini tamamla.",
            ]
        } else {
            lines
        };
        
        let default_steps = [
            ("Görev Analizi", "Proje Analizi", "Sistem Kriteri", "Giriş Doğruluğu"),
            ("Karar Yetkilendirme", "Yetki Seçimi", "Kriter Eşleşmesi", "Matris Onayı"),
            ("Alternatif Analiz", "Alternatif Seçimi", "Puanlama Kriteri", "Risk Derecesi"),
            ("Güvenlik ve Onay", "Snapshot Hazırlığı", "Geri Alma Kriteri", "Kullanıcı Onayı"),
            ("Operasyon Testi", "Uygulama Testleri", "Bütünlük Kontrolü", "Rapor Derleme")
        ];

        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        for (i, line) in lines.into_iter().enumerate() {
            let line_lower = line.to_lowercase();
            
            // Otonom Anahtar Kelime Analizi ve Kategorizasyon
            let (topic, subtopic_default, crit, subcrit) = if line_lower.contains("dosya") || line_lower.contains("file") || line_lower.contains("folder") || line_lower.contains("klasör") || line_lower.contains("yazma") {
                ("Dosya Sistemi Operasyonu", "Dosya/Klasör Modifikasyonu", "Fiziksel Veri Kriteri", "Dosya Değişiklik Kontrolü")
            } else if line_lower.contains("veritabanı") || line_lower.contains("db") || line_lower.contains("sqlite") || line_lower.contains("tablo") || line_lower.contains("sql") {
                ("Veritabanı İşlemi", "SQLite Şema Analizi", "Tablo Bütünlük Kriteri", "Kayıt Doğrulaması")
            } else if line_lower.contains("yetki") || line_lower.contains("onay") || line_lower.contains("matrix") || line_lower.contains("permission") || line_lower.contains("decider") {
                ("Karar Yetkilendirme", "Yetki Seçimi", "Kriter Eşleşmesi", "Matris Onayı")
            } else if line_lower.contains("risk") || line_lower.contains("güvenlik") || line_lower.contains("safety") || line_lower.contains("bariyer") {
                ("Risk Değerlendirmesi", "Kural Analizi", "Güvenlik Bariyeri", "Risk Derecesi Kontrolü")
            } else if line_lower.contains("test") || line_lower.contains("kontrol") || line_lower.contains("bütünlük") || line_lower.contains("doğrulama") {
                ("Operasyon Testi", "Uygulama Testleri", "Bütünlük Kontrolü", "Rapor Derleme")
            } else {
                // Varsayılan Akış (Geriye Dönük Uyumluluk için Rotasyon)
                let step_idx = i % default_steps.len();
                default_steps[step_idx]
            };

            // Derinleştirilmiş Otonom Kategorizasyon ve Kapsam Analizi
            let mut detected_categories = Vec::new();
            if line_lower.contains("canlı sistem") || line_lower.contains("live") || line_lower.contains("production") {
                detected_categories.push("Canlı Sistem");
            }
            if line_lower.contains("api") || line_lower.contains("http") || line_lower.contains("bağlantı") {
                detected_categories.push("API");
            }
            if line_lower.contains("veritabanı") || line_lower.contains("db") || line_lower.contains("sqlite") {
                detected_categories.push("Veritabanı");
            }
            if line_lower.contains("stok") || line_lower.contains("inventory") {
                detected_categories.push("Stok");
            }
            if line_lower.contains("sipariş") || line_lower.contains("order") {
                detected_categories.push("Sipariş");
            }
            if line_lower.contains("üretim") || line_lower.contains("production") {
                detected_categories.push("Üretim");
            }
            if line_lower.contains("müşteri") || line_lower.contains("kvkk") || line_lower.contains("customer") {
                detected_categories.push("Müşteri Verisi");
            }
            if line_lower.contains("finans") || line_lower.contains("para") || line_lower.contains("maliyet") {
                detected_categories.push("Finansal Veri");
            }
            if line_lower.contains("yazma") || line_lower.contains("write") {
                detected_categories.push("Dosya Yazma");
            }
            if line_lower.contains("sil") || line_lower.contains("delete") || line_lower.contains("remove") {
                detected_categories.push("Dosya Silme");
            }
            if line_lower.contains("terminal") || line_lower.contains("komut") || line_lower.contains("cmd") || line_lower.contains("bash") {
                detected_categories.push("Terminal Komutu");
            }
            if line_lower.contains("onay") || line_lower.contains("approval") {
                detected_categories.push("Onay Gereksinimi");
            }
            if line_lower.contains("rollback") || line_lower.contains("yedek") || line_lower.contains("snapshot") || line_lower.contains("geri") {
                detected_categories.push("Rollback/Snapshot");
            }
            if line_lower.contains("test") || line_lower.contains("kontrol") || line_lower.contains("bütünlük") {
                detected_categories.push("Test/Bütünlük");
            }

            // Otonom Ön Risk Etiketi, Muhtemel Konnektör ve Karar Düğümü Analizi
            let risk_pre_label = if line_lower.contains("sil") || line_lower.contains("canlı sistem") || line_lower.contains("terminal") {
                "CRITICAL"
            } else if line_lower.contains("yazma") || line_lower.contains("api") || line_lower.contains("üretim") || line_lower.contains("finans") {
                "HIGH"
            } else if line_lower.contains("veritabanı") || line_lower.contains("stok") || line_lower.contains("müşteri") {
                "MEDIUM"
            } else {
                "LOW"
            };

            let probable_connector = if line_lower.contains("api") || line_lower.contains("http") {
                "api_connector"
            } else if line_lower.contains("veritabanı") || line_lower.contains("db") || line_lower.contains("sqlite") {
                "sqlite_connector"
            } else if line_lower.contains("dosya") || line_lower.contains("folder") || line_lower.contains("klasör") {
                "file_connector"
            } else if line_lower.contains("terminal") || line_lower.contains("komut") {
                "terminal_connector"
            } else if line_lower.contains("rapor") || line_lower.contains("report") {
                "report_manager"
            } else {
                "user_instruction"
            };

            let decision_node_required = if risk_pre_label == "CRITICAL" || risk_pre_label == "HIGH" || line_lower.contains("onay") {
                "Evet (Kullanıcı Onayı Zorunlu)"
            } else {
                "Evet (Otonom Yetkilendirme)"
            };

            let dynamic_subtopic = if detected_categories.is_empty() {
                format!("Dinamik: {} [Ön Risk: {}, Konnektör: {}, Karar Düğümü: {}]", subtopic_default, risk_pre_label, probable_connector, decision_node_required)
            } else {
                format!("Dinamik: {} [Ön Risk: {}, Konnektör: {}, Karar Düğümü: {}]", detected_categories.join(" + "), risk_pre_label, probable_connector, decision_node_required)
            };

            let breakdown = TaskBreakdown {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                parent_id: None,
                level: (i + 1) as i32,
                topic: format!("{} (Adım {})", topic, i + 1),
                subtopic: dynamic_subtopic,
                criterion: crit.to_string(),
                subcriterion: subcrit.to_string(),
                description: Some(line.to_string()),
                risk_pre_label: Some(risk_pre_label.to_string()),
                probable_connector: Some(probable_connector.to_string()),
                decision_node_required: Some(decision_node_required.to_string()),
            };

            conn.execute(
                "INSERT INTO task_breakdown (id, task_id, parent_id, level, topic, subtopic, criterion, subcriterion, description, risk_pre_label, probable_connector, decision_node_required)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    breakdown.id,
                    breakdown.task_id,
                    breakdown.parent_id,
                    breakdown.level,
                    breakdown.topic,
                    breakdown.subtopic,
                    breakdown.criterion,
                    breakdown.subcriterion,
                    breakdown.description,
                    breakdown.risk_pre_label,
                    breakdown.probable_connector,
                    breakdown.decision_node_required
                ],
            ).map_err(|e| e.to_string())?;

            breakdowns.push(breakdown);
        }

        Ok(breakdowns)
    }

    pub fn get_breakdowns(task_id: &str) -> Result<Vec<TaskBreakdown>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, task_id, parent_id, level, topic, subtopic, criterion, subcriterion, description, risk_pre_label, probable_connector, decision_node_required FROM task_breakdown WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![task_id], |row| {
            Ok(TaskBreakdown {
                id: row.get(0)?,
                task_id: row.get(1)?,
                parent_id: row.get(2)?,
                level: row.get(3)?,
                topic: row.get(4)?,
                subtopic: row.get(5)?,
                criterion: row.get(6)?,
                subcriterion: row.get(7)?,
                description: row.get(8)?,
                risk_pre_label: row.get(9)?,
                probable_connector: row.get(10)?,
                decision_node_required: row.get(11)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut breakdowns = Vec::new();
        for row in rows {
            breakdowns.push(row.map_err(|e| e.to_string())?);
        }

        Ok(breakdowns)
    }
}

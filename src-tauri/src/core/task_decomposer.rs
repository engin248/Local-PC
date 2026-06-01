use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        let db = Database::new();
        let mut conn = db.get_connection().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let existing_nodes: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM decision_nodes WHERE task_id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if existing_nodes == 0 {
            tx.execute(
                "DELETE FROM task_breakdown_alternatives WHERE task_id = ?1",
                params![task_id],
            )
            .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM task_breakdown WHERE task_id = ?1", params![task_id])
                .map_err(|e| e.to_string())?;
        }

        let lines: Vec<&str> = user_request
            .split(&['.', '\n', ';'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        const MAX_TOP_LEVEL_LINES: usize = 12;
        let lines = if lines.is_empty() {
            vec![
                "Kullanici talimatini ve hedef etki alanini dogrula",
                "Konu, alt konu, kriter ve alt kriter cikar",
                "Her parca icin real hayat alternatiflerini cikar",
                "Dogru yaklasimi ve en iyi uygulanabilir alternatifi sec",
                "Plan, test ve rollback ile alt birime verilecek operasyon paketini hazirla",
            ]
        } else {
            lines
        };
        let lines: Vec<&str> = lines.into_iter().take(MAX_TOP_LEVEL_LINES).collect();

        let atomic_phases = [
            (
                "Cozumleme",
                "Konu/alt konu/kriter/alt kriter cikarimi",
                "Parcalama Derinligi",
                "En kucuk uygulanabilir parca",
            ),
            (
                "Alternatif Analizi",
                "Real hayat alternatiflerini cikar",
                "Alternatif Kapsami",
                "Her parca icin alternatif",
            ),
            (
                "Dogru Secimi",
                "Kabul edilmis dogrulari belirle",
                "Dogru Yaklasim Kriteri",
                "Dogru once, hiz sonra",
            ),
            (
                "Uygulanabilir Secim",
                "En iyi alternatifi sec",
                "Uygulanabilirlik Kriteri",
                "Teknoloji/etki/rollback uyumu",
            ),
            (
                "Kontrol ve Onay",
                "Kontrol, bagimsiz dogrulama ve son onay",
                "Rol Ayrimi Kriteri",
                "Yapan kontrol edenden ayridir",
            ),
        ];

        let mut sequence: i32 = 1;
        for (i, line) in lines.into_iter().enumerate() {
            let line_lower = line.to_lowercase();
            let (topic, subtopic, criterion, subcriterion) = classify_line(&line_lower);
            let risk_pre_label = classify_risk(&line_lower);
            let probable_connector = classify_connector(&line_lower);
            let decision_node_required = if risk_pre_label == "CRITICAL"
                || risk_pre_label == "HIGH"
                || line_lower.contains("onay")
            {
                "Evet (Kullanici Onayi Zorunlu)"
            } else {
                "Evet (Otonom Yetkilendirme)"
            };

            let parent = TaskBreakdown {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                parent_id: None,
                level: sequence,
                topic: format!("{} (Ana Parca {})", topic, i + 1),
                subtopic: format!(
                    "{} [On Risk: {}; Konnektor: {}; Karar Dugumu: {}]",
                    subtopic, risk_pre_label, probable_connector, decision_node_required
                ),
                criterion: criterion.to_string(),
                subcriterion: subcriterion.to_string(),
                description: Some(line.to_string()),
                risk_pre_label: Some(risk_pre_label.to_string()),
                probable_connector: Some(probable_connector.to_string()),
                decision_node_required: Some(decision_node_required.to_string()),
            };
            insert_breakdown_tx(&tx, &parent)?;
            insert_part_alternatives_tx(&tx, &parent)?;
            let parent_id = parent.id.clone();
            breakdowns.push(parent);
            sequence += 1;

            for (phase, phase_subtopic, phase_criterion, phase_subcriterion) in atomic_phases {
                let child = TaskBreakdown {
                    id: Uuid::new_v4().to_string(),
                    task_id: task_id.to_string(),
                    parent_id: Some(parent_id.clone()),
                    level: sequence,
                    topic: format!("{} / {}", topic, phase),
                    subtopic: format!(
                        "{} [Kaynak Parca: {}; Alternatif Politikasi: her parca icin real hayat alternatifleri]",
                        phase_subtopic,
                        i + 1
                    ),
                    criterion: phase_criterion.to_string(),
                    subcriterion: phase_subcriterion.to_string(),
                    description: Some(format!("{} -> {}", line, phase)),
                    risk_pre_label: Some(risk_pre_label.to_string()),
                    probable_connector: Some(probable_connector.to_string()),
                    decision_node_required: Some(decision_node_required.to_string()),
                };
                insert_breakdown_tx(&tx, &child)?;
                insert_part_alternatives_tx(&tx, &child)?;
                breakdowns.push(child);
                sequence += 1;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(breakdowns)
    }

    pub fn get_breakdowns(task_id: &str) -> Result<Vec<TaskBreakdown>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, task_id, parent_id, level, topic, subtopic, criterion, subcriterion, description, risk_pre_label, probable_connector, decision_node_required FROM task_breakdown WHERE task_id = ?1 ORDER BY level ASC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![task_id], |row| {
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
            })
            .map_err(|e| e.to_string())?;

        let mut breakdowns = Vec::new();
        for row in rows {
            breakdowns.push(row.map_err(|e| e.to_string())?);
        }

        Ok(breakdowns)
    }
}

fn insert_breakdown_tx(
    tx: &rusqlite::Transaction<'_>,
    breakdown: &TaskBreakdown,
) -> Result<(), String> {
    tx.execute(
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
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn insert_part_alternatives_tx(
    tx: &rusqlite::Transaction<'_>,
    breakdown: &TaskBreakdown,
) -> Result<(), String> {
    let alternatives = [
        (
            "Sadece oku ve raporla",
            "Parcayi degistirmeden analiz eder; yan etki olusturmaz.",
            1,
            0,
            "Kabul edilmis dogru yaklasim olarak veri guvenligi ve geri alinabilirlik korunur.",
        ),
        (
            "Uygulama yapma, manuel operasyon plani uret",
            "Parca icin insan tarafindan uygulanabilir plan hazirlar ve otomatik etkiyi durdurur.",
            1,
            0,
            "Yuksek belirsizlik veya kritik riskte en guvenli gercek hayat alternatifidir.",
        ),
        (
            "Onayli, kontrollu ve rollback destekli uygula",
            "Plan, teknoloji, etki alani, kontrol noktasi, test ve rollback tamamlandiktan sonra uygular.",
            1,
            1,
            "Dogru kabul edilmis guvenlik ilkeleriyle birlikte uygulanabilir en iyi secenektir.",
        ),
        (
            "Onaysiz ve rollback'siz dogrudan uygula",
            "Hizli ama kontrolsuz uygulama yapar; standartlara aykiridir ve elenir.",
            0,
            0,
            "Kontrol, test, onay ve rollback kriterlerini saglamadigi icin reddedilir.",
        ),
    ];

    for (idx, (title, description, accepted_correct, selected_best, reason)) in
        alternatives.iter().enumerate()
    {
        tx.execute(
            "INSERT INTO task_breakdown_alternatives
             (id, task_id, breakdown_id, alternative_order, title, description, accepted_correct,
              selected_best, selection_reason, control_criteria, test_criteria, rollback_note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                Uuid::new_v4().to_string(),
                breakdown.task_id,
                breakdown.id,
                (idx + 1) as i32,
                title,
                description,
                accepted_correct,
                selected_best,
                reason,
                format!(
                    "{} / {} / {} / {} kontrol edildi",
                    breakdown.topic, breakdown.subtopic, breakdown.criterion, breakdown.subcriterion
                ),
                "Gercek test kriteri plan paketinde tanimli olmadan yurutme baslamaz",
                "Snapshot/geri alma plani plan paketinde tanimli olmadan yurutme baslamaz"
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn classify_line(line_lower: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    if contains_any(line_lower, &["dosya", "file", "folder", "klasor", "klasör", "yazma"]) {
        (
            "Dosya Sistemi Operasyonu",
            "Dosya/Klasor Modifikasyonu",
            "Fiziksel Veri Kriteri",
            "Dosya Degisiklik Kontrolu",
        )
    } else if contains_any(line_lower, &["veritabani", "veritabanı", "db", "sqlite", "tablo", "sql"]) {
        (
            "Veritabani Islemi",
            "SQLite Sema Analizi",
            "Tablo Butunluk Kriteri",
            "Kayit Dogrulamasi",
        )
    } else if contains_any(line_lower, &["yetki", "onay", "matrix", "permission", "decider"]) {
        (
            "Karar Yetkilendirme",
            "Yetki Secimi",
            "Kriter Eslesmesi",
            "Matris Onayi",
        )
    } else if contains_any(line_lower, &["risk", "guvenlik", "güvenlik", "safety", "bariyer"]) {
        (
            "Risk Degerlendirmesi",
            "Kural Analizi",
            "Guvenlik Bariyeri",
            "Risk Derecesi Kontrolu",
        )
    } else if contains_any(line_lower, &["test", "kontrol", "butunluk", "bütünlük", "dogrulama", "doğrulama"]) {
        (
            "Operasyon Testi",
            "Uygulama Testleri",
            "Butunluk Kontrolu",
            "Rapor Derleme",
        )
    } else {
        (
            "Gorev Analizi",
            "Proje Analizi",
            "Sistem Kriteri",
            "Giris Dogrulugu",
        )
    }
}

fn classify_risk(line_lower: &str) -> &'static str {
    if contains_any(line_lower, &["sil", "delete", "remove", "canli sistem", "canlı sistem", "terminal"]) {
        "CRITICAL"
    } else if contains_any(line_lower, &["yazma", "api", "uretim", "üretim", "finans"]) {
        "HIGH"
    } else if contains_any(line_lower, &["veritabani", "veritabanı", "db", "stok", "musteri", "müşteri"]) {
        "MEDIUM"
    } else {
        "LOW"
    }
}

fn classify_connector(line_lower: &str) -> &'static str {
    if contains_any(line_lower, &["api", "http"]) {
        "api_connector"
    } else if contains_any(line_lower, &["veritabani", "veritabanı", "db", "sqlite"]) {
        "sqlite_connector"
    } else if contains_any(line_lower, &["dosya", "file", "folder", "klasor", "klasör"]) {
        "file_connector"
    } else if contains_any(line_lower, &["terminal", "komut", "cmd"]) {
        "terminal_connector"
    } else if contains_any(line_lower, &["rapor", "report"]) {
        "report_manager"
    } else {
        "user_instruction"
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

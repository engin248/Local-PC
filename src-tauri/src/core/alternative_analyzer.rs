use crate::storage::db::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub risk_score: i32,
    pub dependency_score: i32,
    pub rollback_score: i32,
    pub testability_score: i32,
    pub maintainability_score: i32,
    pub cost_score: i32,
    pub time_score: i32,
    pub user_control_score: i32,
    pub live_impact_score: i32,
    pub data_loss_risk_score: i32,
    pub real_world_basis: String,
    pub ethical_safety_note: String,
    pub selection_reason: String,
    pub accepted_correct_approach_reason: String,
    pub selected_best_option_reason: String,
    pub selected: i32,
    pub reason: Option<String>,
}

pub struct AlternativeAnalyzer;

impl AlternativeAnalyzer {
    pub fn generate_alternatives(node_id: &str) -> Result<Vec<Alternative>, String> {
        let db = Database::new();
        let conn = db.get_connection().map_err(|e| e.to_string())?;

        // Retrieve task context dynamically via node_id
        let (task_id, user_request, task_risk): (String, String, String) = conn
            .query_row(
                "SELECT t.id, t.user_request, t.risk_level FROM tasks t 
             JOIN decision_nodes n ON t.id = n.task_id 
             WHERE n.id = ?1",
                params![node_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| {
                format!(
                "Alternatif analizi için görev metni bulunamadı veya okunamadı (node_id={}): {}",
                node_id, e
            )
            })?;

        let keyword = if user_request.to_lowercase().contains("dosya")
            || user_request.to_lowercase().contains("file")
        {
            "Dosya Modifikasyonu"
        } else if user_request.to_lowercase().contains("veritabanı")
            || user_request.to_lowercase().contains("db")
        {
            "Veritabanı İşlemi"
        } else {
            "Sistem Operasyonu"
        };
        let risk = task_risk.to_lowercase();
        let plan = crate::core::planning_gate::PlanningGate::load_plan(&task_id).ok();
        let impact_area = plan
            .as_ref()
            .map(|p| p.impact_area.as_str())
            .unwrap_or("tanimsiz etki alani");
        let technology = plan
            .as_ref()
            .map(|p| p.technology_selection.as_str())
            .unwrap_or("mevcut teknoloji");
        let correct_reason = plan
            .as_ref()
            .map(|p| p.accepted_correct_approach_reason.clone())
            .unwrap_or_else(|| {
                "Genel dogru yaklasim kullanici kontrolu, veri gizliligi, rollback ve test edilebilirligi korur.".to_string()
            });
        let best_reason = plan
            .as_ref()
            .map(|p| p.selected_best_option_reason.clone())
            .unwrap_or_else(|| {
                "En iyi uygulanabilir secenek mevcut sistemle uyumlu, geri alinabilir ve test edilebilir olandir.".to_string()
            });

        let alts = vec![
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Onayli ve rollback destekli {} uygulamasi", keyword),
                description: format!("{} hedefinde {} kullanilarak once snapshot alinir, kullanici onayi beklenir, ardindan Test Gate ile dogrulama yapilir.", impact_area, technology),
                pros_json: "[\"Geri alinabilir\", \"Kullanici kontrolu yuksek\", \"Test Gate ile dogrulanabilir\"]".to_string(),
                cons_json: "[\"Onay bekler\", \"Snapshot ve test suresi gerektirir\"]".to_string(),
                accuracy_score: 9,
                safety_score: 10,
                risk_score: 2,
                dependency_score: 10,
                rollback_score: 10,
                testability_score: 10,
                maintainability_score: 9,
                cost_score: 10,
                time_score: 7,
                user_control_score: 10,
                live_impact_score: 2,
                data_loss_risk_score: 1,
                real_world_basis: "Degisiklik oncesi snapshot, manuel onay ve test sonrasi tamamlama yaygin guvenli operasyon standardidir.".to_string(),
                ethical_safety_note: "Kullanici iradesi korunur; AI ciktisi onay yerine gecmez.".to_string(),
                selection_reason: best_reason.clone(),
                accepted_correct_approach_reason: correct_reason.clone(),
                selected_best_option_reason: best_reason.clone(),
                selected: 1,
                reason: Some(format!("{} icin en guvenli, geri alinabilir ve kontrollu yontem.", keyword)),
            },
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Sadece oku ve {} raporu hazirla", keyword),
                description: format!("{} alanina yazma yapmadan okuma, analiz ve raporlama yapilir.", impact_area),
                pros_json: "[\"Sifir yazma riski\", \"Hizli denetim\", \"Kullanici kararini korur\"]".to_string(),
                cons_json: "[\"Eksiklerin uygulanmasi icin ayri onayli gorev gerekir\"]".to_string(),
                accuracy_score: 8,
                safety_score: 10,
                risk_score: 1,
                dependency_score: 10,
                rollback_score: 10,
                testability_score: 9,
                maintainability_score: 8,
                cost_score: 9,
                time_score: 8,
                user_control_score: 10,
                live_impact_score: 0,
                data_loss_risk_score: 0,
                real_world_basis: "Denetim ve kesif asamalarinda read-only analiz, yan etki riskini azaltan kabul gormus yaklasimdir.".to_string(),
                ethical_safety_note: "Veri degistirmez ve kullanici onayi gerektiren islemleri ayrica raporlar.".to_string(),
                selection_reason: "Guvenli kesif icin uygundur ancak uygulama hedefini tek basina tamamlamaz.".to_string(),
                accepted_correct_approach_reason: correct_reason.clone(),
                selected_best_option_reason: best_reason.clone(),
                selected: 0,
                reason: Some("Read-only guvenli alternatif olarak zorunlu tutuldu.".to_string()),
            },
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Uygulama yapma, manuel {} plani uret", keyword),
                description: format!("{} icin otomatik uygulama durdurulur; kullaniciya manuel, onaylanabilir operasyon plani sunulur.", keyword),
                pros_json: "[\"High/Critical riskte guvenli\", \"Kullanici son karari verir\", \"Plan olgunlasir\"]".to_string(),
                cons_json: "[\"Otomatik tamamlanma saglamaz\", \"Ek insan karari gerekir\"]".to_string(),
                accuracy_score: 9,
                safety_score: 10,
                risk_score: if risk == "critical" { 1 } else { 2 },
                dependency_score: 10,
                rollback_score: 9,
                testability_score: 8,
                maintainability_score: 8,
                cost_score: 7,
                time_score: 5,
                user_control_score: 10,
                live_impact_score: 0,
                data_loss_risk_score: 0,
                real_world_basis: "Yuksek riskli operasyonlarda otomasyonu durdurup manuel plan istemek guvenlik ve uyum acisindan tercih edilir.".to_string(),
                ethical_safety_note: "Kritik etki alanlarinda hiz yerine insan denetimi oncelenir.".to_string(),
                selection_reason: "Risk cok yuksekse tercih edilecek koruyucu alternatif.".to_string(),
                accepted_correct_approach_reason: correct_reason.clone(),
                selected_best_option_reason: best_reason.clone(),
                selected: 0,
                reason: Some("High/Critical riskler icin manuel plan alternatifi.".to_string()),
            },
            Alternative {
                id: Uuid::new_v4().to_string(),
                decision_node_id: node_id.to_string(),
                title: format!("Onaysiz dogrudan {} uygulama", keyword),
                description: format!("{} uzerinde AI veya connector ciktisina guvenerek onaysiz ve rollback plansiz islem yapilir.", keyword),
                pros_json: "[\"Cok hizli\"]".to_string(),
                cons_json: "[\"Yuksek veri kaybi riski\", \"Kullanici iradesini devre disi birakir\", \"Rollback zayif\"]".to_string(),
                accuracy_score: 4,
                safety_score: 1,
                risk_score: 10,
                dependency_score: 8,
                rollback_score: 1,
                testability_score: 2,
                maintainability_score: 5,
                cost_score: 10,
                time_score: 10,
                user_control_score: 1,
                live_impact_score: 9,
                data_loss_risk_score: 9,
                real_world_basis: "Bu alternatif operasyonel guvenlik standartlarina aykiridir ve karsilastirma icin elenir.".to_string(),
                ethical_safety_note: "AI ciktisinin insan onayi yerine gecmesi kabul edilmez.".to_string(),
                selection_reason: "Guvenlik ve kullanici kontrolu kriterlerini saglamadigi icin elendi.".to_string(),
                accepted_correct_approach_reason: correct_reason.clone(),
                selected_best_option_reason: best_reason.clone(),
                selected: 0,
                reason: Some("Onaysiz ve rollback plansiz oldugu icin elendi.".to_string()),
            }
        ];

        for alt in &alts {
            conn.execute(
                "INSERT INTO alternatives (id, decision_node_id, title, description, pros_json, cons_json, accuracy_score, safety_score, dependency_score, rollback_score, maintainability_score, cost_score, time_score, user_control_score, live_impact_score, data_loss_risk_score, real_world_basis, testability_score, ethical_safety_note, selection_reason, accepted_correct_approach_reason, selected_best_option_reason, selected, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
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
                    alt.real_world_basis,
                    alt.testability_score,
                    alt.ethical_safety_note,
                    alt.selection_reason,
                    alt.accepted_correct_approach_reason,
                    alt.selected_best_option_reason,
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
        let mut stmt = conn.prepare("SELECT id, decision_node_id, title, description, pros_json, cons_json, accuracy_score, safety_score, dependency_score, rollback_score, maintainability_score, cost_score, time_score, user_control_score, live_impact_score, data_loss_risk_score, COALESCE(real_world_basis, ''), COALESCE(testability_score, 0), COALESCE(ethical_safety_note, ''), COALESCE(selection_reason, ''), COALESCE(accepted_correct_approach_reason, ''), COALESCE(selected_best_option_reason, ''), selected, reason FROM alternatives WHERE decision_node_id = ?1")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![node_id], |row| {
                Ok(Alternative {
                    id: row.get(0)?,
                    decision_node_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    pros_json: row.get(4)?,
                    cons_json: row.get(5)?,
                    accuracy_score: row.get(6)?,
                    safety_score: row.get(7)?,
                    risk_score: 10 - row.get::<_, i32>(7)?,
                    dependency_score: row.get(8)?,
                    rollback_score: row.get(9)?,
                    maintainability_score: row.get(10)?,
                    cost_score: row.get(11)?,
                    time_score: row.get(12)?,
                    user_control_score: row.get(13)?,
                    live_impact_score: row.get(14)?,
                    data_loss_risk_score: row.get(15)?,
                    real_world_basis: row.get(16)?,
                    testability_score: row.get(17)?,
                    ethical_safety_note: row.get(18)?,
                    selection_reason: row.get(19)?,
                    accepted_correct_approach_reason: row.get(20)?,
                    selected_best_option_reason: row.get(21)?,
                    selected: row.get(22)?,
                    reason: row.get(23)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut alts = Vec::new();
        for row in rows {
            alts.push(row.map_err(|e| e.to_string())?);
        }

        Ok(alts)
    }
}

use crate::core::asker_motoru_bridge::AskerMotoruBridge;
use crate::core::dependency_analyzer::DependencyAnalyzer;
use crate::core::skill_library::SkillLibrary;
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerModuleRegistryConfig {
    pub schema_version: u32,
    pub expected_module_total: u32,
    pub inventory_files: Vec<String>,
    pub sqlite: Option<SqliteModuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteModuleConfig {
    pub module_table_candidates: Vec<String>,
    pub skill_table: String,
    pub module_id_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerModuleRecord {
    pub module_id: String,
    pub name: String,
    pub duty: Option<String>,
    pub skills: Vec<String>,
    pub system_skills: Vec<String>,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskerModuleSummary {
    pub expected_module_total: u32,
    pub registered_module_count: u32,
    pub active_in_panel_count: u32,
    pub skill_count: u32,
    pub source_kind: String,
    pub source_path: Option<String>,
    pub inventory_match: bool,
    pub last_error: Option<String>,
}

pub struct AskerModuleRegistry;

impl AskerModuleRegistry {
    pub fn load_config() -> AskerModuleRegistryConfig {
        let default = AskerModuleRegistryConfig {
            schema_version: 1,
            expected_module_total: 314,
            inventory_files: vec![
                "UZMAN_HAVUZU.json".to_string(),
                "module_inventory.json".to_string(),
            ],
            sqlite: Some(SqliteModuleConfig {
                module_table_candidates: vec![
                    "modules".to_string(),
                    "swarm_modules".to_string(),
                    "asker_modules".to_string(),
                ],
                skill_table: "skills".to_string(),
                module_id_columns: vec![
                    "module_id".to_string(),
                    "modul_id".to_string(),
                    "module_no".to_string(),
                ],
            }),
        };
        let Ok(path) = DependencyAnalyzer::get_config_path("asker_module_registry.json") else {
            return default;
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or(default)
    }

    pub fn summary() -> AskerModuleSummary {
        let config = Self::load_config();
        let mut modules = Vec::new();
        let mut source_kind = "unavailable".to_string();
        let mut source_path = None;
        let mut last_error = None;
        let mut skill_count = 0u32;

        if let Some((path, sqlite_modules, skills)) = Self::load_from_sqlite(&config) {
            source_kind = "sqlite".to_string();
            source_path = Some(path.display().to_string());
            modules.extend(sqlite_modules);
            skill_count = skills;
        }

        if modules.is_empty() {
            if let Some((path, json_modules)) = Self::load_from_asker_roots(&config) {
                source_kind = "json".to_string();
                source_path = Some(path);
                modules.extend(json_modules);
            } else if last_error.is_none() {
                last_error = Some(
                    "Modül envanteri tablosu veya UZMAN_HAVUZU.json henüz bağlı değil.".to_string(),
                );
            }
        }

        let registered_module_count = modules.len() as u32;
        let active_in_panel_count = Self::count_active_panel_platforms();
        let inventory_match = registered_module_count == config.expected_module_total;

        AskerModuleSummary {
            expected_module_total: config.expected_module_total,
            registered_module_count,
            active_in_panel_count,
            skill_count,
            source_kind,
            source_path,
            inventory_match,
            last_error,
        }
    }

    pub fn list_modules(limit: usize) -> Result<Vec<AskerModuleRecord>, String> {
        let config = Self::load_config();
        let mut modules = Vec::new();

        if let Some((_, sqlite_modules, _)) = Self::load_from_sqlite(&config) {
            modules.extend(sqlite_modules);
        }
        if modules.is_empty() {
            if let Some((_, json_modules)) = Self::load_from_asker_roots(&config) {
                modules.extend(json_modules);
            }
        }

        modules.truncate(limit);
        Ok(modules)
    }

    pub fn module_skills(module_id: &str) -> Result<Vec<String>, String> {
        let config = Self::load_config();
        let Some(path) = SkillLibrary::resolve_db_path() else {
            return Err("Beceri kütüphanesi SQLite yolu bulunamadı.".to_string());
        };
        let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
        let skill_table = config
            .sqlite
            .as_ref()
            .map(|s| s.skill_table.as_str())
            .unwrap_or("skills");

        for column in config
            .sqlite
            .as_ref()
            .map(|s| s.module_id_columns.clone())
            .unwrap_or_else(|| vec!["module_id".to_string()])
        {
            let sql = format!(
                "SELECT name FROM {skill_table} WHERE {column} = ?1 LIMIT 200"
            );
            if let Ok(mut stmt) = conn.prepare(&sql) {
                let rows = stmt.query_map([module_id], |row| row.get::<_, String>(0));
                if let Ok(rows) = rows {
                    let mut skills = Vec::new();
                    for row in rows.flatten() {
                        skills.push(row);
                    }
                    if !skills.is_empty() {
                        return Ok(skills);
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    fn load_from_sqlite(
        config: &AskerModuleRegistryConfig,
    ) -> Option<(PathBuf, Vec<AskerModuleRecord>, u32)> {
        let path = SkillLibrary::resolve_db_path()?;
        let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
        let sqlite_cfg = config.sqlite.as_ref()?;
        let mut modules = Vec::new();

        for table in &sqlite_cfg.module_table_candidates {
            if let Ok(mut stmt) =
                conn.prepare(&format!("SELECT * FROM {table} LIMIT 500"))
            {
                let names: Vec<String> = stmt
                    .column_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                if names.is_empty() {
                    continue;
                }
                let rows = stmt.query_map([], |row| {
                    let mut map = HashMap::new();
                    for (idx, name) in names.iter().enumerate() {
                        let value: String = row.get(idx).unwrap_or_default();
                        map.insert(name.clone(), value);
                    }
                    Ok(map)
                });
                if let Ok(rows) = rows {
                    for row in rows.flatten() {
                        modules.push(Self::map_record(&row, "sqlite", table));
                    }
                    if !modules.is_empty() {
                        let skill_count = conn
                            .query_row(
                                &format!("SELECT COUNT(*) FROM {}", sqlite_cfg.skill_table),
                                [],
                                |row| row.get::<_, i64>(0),
                            )
                            .unwrap_or(0) as u32;
                        return Some((path, modules, skill_count));
                    }
                }
            }
        }

        // skills tablosundan modül bazlı gruplama
        for column in &sqlite_cfg.module_id_columns {
            let sql = format!(
                "SELECT {column}, COUNT(*) as cnt FROM {} WHERE {column} IS NOT NULL AND TRIM({column}) != '' GROUP BY {column} ORDER BY {column} LIMIT 500",
                sqlite_cfg.skill_table,
                column = column
            );
            if let Ok(mut stmt) = conn.prepare(&sql) {
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                    ))
                });
                if let Ok(rows) = rows {
                    for row in rows.flatten() {
                        let (module_id, count) = row;
                        modules.push(AskerModuleRecord {
                            module_id: module_id.clone(),
                            name: format!("Modül {module_id}"),
                            duty: Some(format!("{count} beceri kayıtlı")),
                            skills: Vec::new(),
                            system_skills: Vec::new(),
                            status: "registered".to_string(),
                            source: format!("sqlite:{}/{}", sqlite_cfg.skill_table, column),
                        });
                    }
                    if !modules.is_empty() {
                        let skill_count = conn
                            .query_row(
                                &format!("SELECT COUNT(*) FROM {}", sqlite_cfg.skill_table),
                                [],
                                |row| row.get::<_, i64>(0),
                            )
                            .unwrap_or(0) as u32;
                        return Some((path, modules, skill_count));
                    }
                }
            }
        }

        None
    }

    fn load_from_asker_roots(config: &AskerModuleRegistryConfig) -> Option<(String, Vec<AskerModuleRecord>)> {
        let report = AskerMotoruBridge::scan_status_files();
        for file in report.files {
            if !file.exists {
                continue;
            }
            let file_name = Path::new(&file.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !config.inventory_files.iter().any(|f| file_name.contains(f.trim_end_matches(".md"))) {
                continue;
            }
            let Ok(content) = fs::read_to_string(&file.path) else {
                continue;
            };
            let modules = Self::parse_inventory_content(&content, &file.path);
            if !modules.is_empty() {
                return Some((file.path, modules));
            }
        }

        for (_, root) in Self::asker_roots() {
            for name in &config.inventory_files {
                let path = root.join(name);
                if !path.exists() {
                    continue;
                }
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let modules = Self::parse_inventory_content(&content, &path.display().to_string());
                if !modules.is_empty() {
                    return Some((path.display().to_string(), modules));
                }
            }
        }
        None
    }

    fn parse_inventory_content(content: &str, source: &str) -> Vec<AskerModuleRecord> {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            return Self::parse_json_inventory(&value, source);
        }
        // markdown envanter: satır başı modül kodları
        content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("### Modül") || trimmed.starts_with("## Modül") {
                    let id = trimmed
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("bilinmiyor")
                        .trim_matches(|c| c == ':' || c == '.')
                        .to_string();
                    Some(AskerModuleRecord {
                        module_id: id.clone(),
                        name: trimmed.to_string(),
                        duty: None,
                        skills: Vec::new(),
                        system_skills: Vec::new(),
                        status: "manifest".to_string(),
                        source: format!("markdown:{source}"),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn parse_json_inventory(value: &serde_json::Value, source: &str) -> Vec<AskerModuleRecord> {
        let items = if let Some(arr) = value.as_array() {
            arr.clone()
        } else if let Some(arr) = value.get("modules").and_then(|v| v.as_array()) {
            arr.clone()
        } else if let Some(arr) = value.get("uzmanlar").and_then(|v| v.as_array()) {
            arr.clone()
        } else if let Some(obj) = value.as_object() {
            obj.values().cloned().collect()
        } else {
            Vec::new()
        };

        items
            .into_iter()
            .filter_map(|item| {
                let module_id = item
                    .get("module_id")
                    .or_else(|| item.get("modul_id"))
                    .or_else(|| item.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if module_id.is_empty() {
                    return None;
                }
                let name = item
                    .get("name")
                    .or_else(|| item.get("title"))
                    .or_else(|| item.get("uzmanlik"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&module_id)
                    .to_string();
                let duty = item
                    .get("duty")
                    .or_else(|| item.get("gorev"))
                    .or_else(|| item.get("role"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                let skills = Self::string_list(item.get("skills").or_else(|| item.get("beceriler")));
                let system_skills =
                    Self::string_list(item.get("system_skills").or_else(|| item.get("sistem_becerileri")));
                let status = item
                    .get("status")
                    .or_else(|| item.get("durum"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("registered")
                    .to_string();
                Some(AskerModuleRecord {
                    module_id,
                    name,
                    duty,
                    skills,
                    system_skills,
                    status,
                    source: format!("json:{source}"),
                })
            })
            .collect()
    }

    fn string_list(value: Option<&serde_json::Value>) -> Vec<String> {
        match value {
            Some(serde_json::Value::Array(items)) => items
                .iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect(),
            Some(serde_json::Value::String(text)) => vec![text.clone()],
            _ => Vec::new(),
        }
    }

    fn map_record(row: &HashMap<String, String>, source: &str, table: &str) -> AskerModuleRecord {
        let pick = |keys: &[&str]| -> String {
            keys.iter()
                .find_map(|key| row.get(*key))
                .cloned()
                .unwrap_or_else(|| "bilinmiyor".to_string())
        };
        AskerModuleRecord {
            module_id: pick(&["module_id", "modul_id", "id", "module_no", "modul_no"]),
            name: pick(&["name", "title", "module_name", "modul_adi"]),
            duty: row.get("duty").or_else(|| row.get("gorev")).cloned(),
            skills: row
                .get("skills")
                .map(|v| v.split(',').map(str::trim).map(ToString::to_string).collect())
                .unwrap_or_default(),
            system_skills: row
                .get("system_skills")
                .map(|v| v.split(',').map(str::trim).map(ToString::to_string).collect())
                .unwrap_or_default(),
            status: pick(&["status", "durum"]),
            source: format!("{source}:{table}"),
        }
    }

    fn count_active_panel_platforms() -> u32 {
        let Ok(path) = DependencyAnalyzer::get_config_path("ai_workspaces.json") else {
            return 0;
        };
        let Ok(data) = fs::read_to_string(path) else {
            return 0;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else {
            return 0;
        };
        value
            .get("platforms")
            .and_then(|v| v.as_object())
            .map(|obj| obj.len() as u32)
            .unwrap_or(0)
    }

    fn asker_roots() -> Vec<(String, PathBuf)> {
        AskerMotoruBridge::scan_status_files()
            .roots_checked
            .into_iter()
            .map(|path| ("root".to_string(), PathBuf::from(path)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_keeps_expected_module_total_314() {
        let config = AskerModuleRegistry::load_config();
        assert_eq!(config.expected_module_total, 314);
    }

    #[test]
    fn parse_json_inventory_reads_duty_and_skills() {
        let raw = r#"{
            "modules": [
                {
                    "module_id": "042",
                    "name": "Taktik Analist",
                    "duty": "Saha verisi sentezi",
                    "skills": ["veri_analizi", "raporlama"],
                    "system_skills": ["sqlite_okuma"],
                    "status": "registered"
                }
            ]
        }"#;
        let value: serde_json::Value = serde_json::from_str(raw).unwrap();
        let modules = AskerModuleRegistry::parse_json_inventory(&value, "test.json");
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].module_id, "042");
        assert_eq!(modules[0].duty.as_deref(), Some("Saha verisi sentezi"));
        assert_eq!(modules[0].skills, vec!["veri_analizi", "raporlama"]);
        assert_eq!(modules[0].system_skills, vec!["sqlite_okuma"]);
    }

    #[test]
    fn summary_reports_expected_total_even_when_inventory_missing() {
        let summary = AskerModuleRegistry::summary();
        assert_eq!(summary.expected_module_total, 314);
    }
}

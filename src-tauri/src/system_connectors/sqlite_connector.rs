use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use rusqlite::Connection;
use serde_json::{json, Value};

pub struct SqliteConnector {
    pub name: String,
}

impl SystemConnector for SqliteConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        // Target format: "db_path|sql_query"
        let parts: Vec<&str> = target.split('|').collect();
        if parts.len() < 2 {
            return Err("SQLite okuma biçimi geçersiz! Format: 'db_yolu|sql_sorgusu'".to_string());
        }
        let db_path = parts[0];
        let query = parts[1];

        let conn =
            Connection::open(db_path).map_err(|e| format!("Hedef veritabanı açılamadı: {}", e))?;
        let mut stmt = conn
            .prepare(query)
            .map_err(|e| format!("Sorgu hazırlama hatası: {}", e))?;

        let col_count = stmt.column_count();
        let mut col_names = Vec::new();
        for i in 0..col_count {
            col_names.push(stmt.column_name(i).map_err(|e| e.to_string())?.to_string());
        }

        let mut rows = stmt
            .query([])
            .map_err(|e| format!("Sorgu yürütme hatası: {}", e))?;
        let mut result_list = Vec::new();

        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let mut row_json = serde_json::Map::new();
            for (idx, name) in col_names.iter().enumerate() {
                let value_ref = row.get_ref(idx).map_err(|e| e.to_string())?;
                let val = match value_ref {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => json!(i),
                    rusqlite::types::ValueRef::Real(r) => json!(r),
                    rusqlite::types::ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
                    rusqlite::types::ValueRef::Blob(b) => {
                        json!(format!("BLOB ({} bytes)", b.len()))
                    }
                };
                row_json.insert(name.clone(), val);
            }
            result_list.push(Value::Object(row_json));
        }

        serde_json::to_string(&result_list).map_err(|e| e.to_string())
    }

    fn execute_write(&self, target: &str, data: &str) -> Result<(), String> {
        // target is db_path, payload is raw SQL query
        let (context, payload) = decode_write_request("sqlite_write", data)?;
        require_authorized_write(&context)?;

        let target_conn =
            Connection::open(target).map_err(|e| format!("Hedef veritabanı açılamadı: {}", e))?;
        target_conn
            .execute(&payload, [])
            .map_err(|e| format!("SQLite yazma sorgusu yürütülemedi: {}", e))?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

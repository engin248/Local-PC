use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use serde_json::json;
use std::fs;

pub struct FolderConnector {
    pub name: String,
}

impl SystemConnector for FolderConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        let entries =
            fs::read_dir(target).map_err(|e| format!("Dizin okuma hatası ({}): {}", target, e))?;

        let mut list = Vec::new();
        for entry in entries.flatten() {
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            list.push(json!({
                "name": entry.file_name().to_string_lossy(),
                "is_dir": metadata.is_dir(),
                "size": metadata.len()
            }));
        }

        serde_json::to_string(&list).map_err(|e| e.to_string())
    }

    fn execute_write(&self, target: &str, data: &str) -> Result<(), String> {
        let (context, _) = decode_write_request("write_folder", data)?;
        require_authorized_write(&context)?;

        fs::create_dir_all(target)
            .map_err(|e| format!("Fiziksel klasör oluşturma hatası ({}): {}", target, e))?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use std::fs;

pub struct FileConnector {
    pub name: String,
}

impl SystemConnector for FileConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        fs::read_to_string(target).map_err(|e| format!("Dosya okuma hatası ({}): {}", target, e))
    }

    fn execute_write(&self, target: &str, data: &str) -> Result<(), String> {
        let (context, payload) = decode_write_request("write_file", data)?;
        require_authorized_write(&context)?;

        fs::write(target, payload)
            .map_err(|e| format!("Fiziksel dosya yazma hatası ({}): {}", target, e))?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_file_write_without_valid_approval() {
        let connector = FileConnector {
            name: "test_file_connector".to_string(),
        };
        let target = std::env::temp_dir().join("unauthorized_panel_write_test.txt");
        let _ = std::fs::remove_file(&target);
        let target_text = target.to_string_lossy().into_owned();

        let missing_context = connector.execute_write(&target_text, "plain payload");
        assert!(missing_context.is_err());
        assert!(!target.exists());

        let no_db_approval = serde_json::json!({
            "approval_context": {
                "task_id": "installer_security_test_task",
                "decision_node_id": "installer_security_test_node",
                "action": "write_file",
                "risk_level": "high"
            },
            "payload": "unauthorized payload"
        })
        .to_string();

        let blocked = connector.execute_write(&target_text, &no_db_approval);
        assert!(blocked.is_err());
        assert!(!target.exists());
    }
}

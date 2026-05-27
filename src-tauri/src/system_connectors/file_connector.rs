use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use std::fs;

pub struct FileConnector {
    pub name: String,
}

impl SystemConnector for FileConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        fs::read_to_string(target)
            .map_err(|e| format!("Dosya okuma hatası ({}): {}", target, e))
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

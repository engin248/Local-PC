use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use std::process::Command;

pub struct TerminalConnector {
    pub name: String,
}

impl SystemConnector for TerminalConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        let output = Command::new("powershell")
            .args(["-Command", target])
            .output()
            .map_err(|e| format!("PowerShell okuma komutu çalıştırılamadı: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Komut hatayla sonuçlandı: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn execute_write(&self, target: &str, data: &str) -> Result<(), String> {
        let (context, _) = decode_write_request("terminal_command", data)?;
        require_authorized_write(&context)?;

        let output = Command::new("powershell")
            .args(["-Command", target])
            .output()
            .map_err(|e| format!("PowerShell yazma komutu çalıştırılamadı: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Komut hatayla sonuçlandı: {}", stderr));
        }

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

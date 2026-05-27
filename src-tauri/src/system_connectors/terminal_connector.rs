use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use std::process::Command;

pub struct TerminalConnector {
    pub name: String,
}

impl SystemConnector for TerminalConnector {
    fn execute_read(&self, _target: &str) -> Result<String, String> {
        Err(
            "HATA: Terminal connector read-only health-check dışında komut çalıştıramaz. Komut yürütme yalnızca terminal_command approval context, rollback planı ve Test Gate ile execute_write üzerinden yapılır."
                .to_string(),
        )
    }

    fn execute_write(&self, _target: &str, data: &str) -> Result<(), String> {
        let (context, command) = decode_write_request("terminal_command", data)?;
        require_authorized_write(&context)?;

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                command.as_str(),
            ])
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

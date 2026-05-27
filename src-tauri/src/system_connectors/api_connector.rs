use crate::system_connectors::connector_base::{
    decode_write_request, require_authorized_write, SystemConnector,
};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct ApiConnector {
    pub name: String,
}

impl SystemConnector for ApiConnector {
    fn execute_read(&self, target: &str) -> Result<String, String> {
        // Target format: "host:port|path"
        let parts: Vec<&str> = target.split('|').collect();
        let (addr, path) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else {
            (target, "/")
        };

        let socket_addrs: Vec<_> = addr
            .to_socket_addrs()
            .map_err(|e| format!("Adres çözümleme hatası ({}): {}", addr, e))?
            .collect();
        if socket_addrs.is_empty() {
            return Err(format!("Adres bulunamadı: {}", addr));
        }

        let mut stream = TcpStream::connect_timeout(&socket_addrs[0], Duration::from_secs(3))
            .map_err(|e| format!("API bağlantı hatası ({}): {}", addr, e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, addr
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| e.to_string())?;

        // Extract HTTP body
        if let Some(pos) = response.find("\r\n\r\n") {
            Ok(response[pos + 4..].to_string())
        } else {
            Ok(response)
        }
    }

    fn execute_write(&self, target: &str, data: &str) -> Result<(), String> {
        // target is host:port|path, payload is HTTP payload
        let (context, payload) = decode_write_request("api_write", data)?;
        require_authorized_write(&context)?;

        let parts: Vec<&str> = target.split('|').collect();
        let (addr, path) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else {
            (target, "/")
        };

        let socket_addrs: Vec<_> = addr
            .to_socket_addrs()
            .map_err(|e| format!("Adres çözümleme hatası ({}): {}", addr, e))?
            .collect();
        if socket_addrs.is_empty() {
            return Err(format!("Adres bulunamadı: {}", addr));
        }

        let mut stream = TcpStream::connect_timeout(&socket_addrs[0], Duration::from_secs(3))
            .map_err(|e| format!("API bağlantı hatası ({}): {}", addr, e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;

        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            path, addr, payload.len(), payload
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

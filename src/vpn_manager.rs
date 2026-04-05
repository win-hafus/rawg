use crate::{ConnectionStatus, ServerConfig};
// use dirs;
use secrecy::{ExposeSecret, SecretString};
use std::fs;
use std::io;
use std::io::Write;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

pub struct VpnManager;

impl VpnManager {
    pub fn connect(path: PathBuf, password: &SecretString) -> ConnectionStatus {
        let Ok(mut child) = Command::new("sudo")
            .args([
                "-S",
                "-p",
                "",
                "awg-quick",
                "up",
                path.to_str().expect("Not valid path!"),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return ConnectionStatus::Disconnected;
        };

        if let Some(mut stdin) = child.stdin.take() {
            if writeln!(stdin, "{}", password.expose_secret()).is_err() {
                return ConnectionStatus::Disconnected;
            }
        }

        match child.wait_with_output() {
            Ok(output) if output.status.success() => ConnectionStatus::Connected,
            _ => ConnectionStatus::Disconnected,
        }
    }

    pub fn disconnect(path: PathBuf, password: &SecretString) -> ConnectionStatus {
        let Ok(mut child) = Command::new("sudo")
            .args([
                "-S",
                "-p",
                "",
                "awg-quick",
                "down",
                path.to_str().expect("Not valid path!"),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return ConnectionStatus::Connected;
        };

        if let Some(mut stdin) = child.stdin.take() {
            if writeln!(stdin, "{}", password.expose_secret()).is_err() {
                return ConnectionStatus::Connected;
            }
        }

        match child.wait_with_output() {
            Ok(output) if output.status.success() => ConnectionStatus::Disconnected,
            _ => ConnectionStatus::Connected,
        }
    }

    pub fn save_config(path: PathBuf, name: String) -> Result<(), io::Error> {
        let file_content = fs::read_to_string(&path)?;
        let new_file_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(format!(".local/share/rawg/{}.conf", name));

        let config_dir = dirs::home_dir()
            .expect("Could not find home directory")
            .join(".local/share/rawg");

        fs::create_dir_all(config_dir)?;
        fs::write(new_file_path, file_content)?;

        Ok(())
    }

    pub fn remove_config(name: String) -> Result<(), io::Error> {
        let file_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(format!(".local/share/rawg/{}.conf", name));
        fs::remove_file(file_path)?;

        Ok(())
    }

    pub fn validate_config(path: &PathBuf) -> bool {
        let Ok(content) = fs::read_to_string(path) else {
            return false;
        };

        // Обязательные секции для WireGuard/AmneziaWG конфига
        let has_interface = content.contains("[Interface]");
        let has_peer = content.contains("[Peer]");

        // Обязательные поля в [Interface]
        let has_private_key = content.contains("PrivateKey");
        let has_address = content.contains("Address");

        // Обязательные поля в [Peer]
        let has_public_key = content.contains("PublicKey");
        let has_endpoint = content.contains("Endpoint");

        has_interface
            && has_peer
            && has_private_key
            && has_address
            && has_public_key
            && has_endpoint
    }
    pub fn check_connection_status(name: &str) -> ConnectionStatus {
        let output = Command::new("ip").args(["link", "show", name]).output();

        match output {
            Ok(out) if out.status.success() => ConnectionStatus::Connected,
            _ => ConnectionStatus::Disconnected,
        }
    }
}

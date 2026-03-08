use crate::{ConnectionStatus, ServerConfig};
use dirs;
use std::{fs, io, process::Command};

pub struct VpnManager;

impl VpnManager {
    pub fn new() -> Self {
        VpnManager
    }

    pub fn connect(name: &str) -> ConnectionStatus {
        let output = Command::new("sudo")
            .args(["awg-quick", "up", name])
            .output()
            .expect("Something went wrong!");
        if output.status.success() {
            ConnectionStatus::Connected(format!("Connected to {}", name))
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            panic!("{}", error)
        }
    }
    pub fn disconnect(name: &str) -> ConnectionStatus {
        let output = Command::new("sudo")
            .args(["awg-quick", "down", name])
            .output()
            .expect("Something went wrong!");
        if output.status.success() {
            ConnectionStatus::Disconnected
        } else {
            let error = String::from_utf8_lossy(&output.stderr).to_string();
            panic!("{}", error)
        }
    }

    pub fn save_config(path: String, name: String) -> Result<(), io::Error> {
        let file_content = fs::read_to_string(path)?;
        let new_file_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(format!(".local/share/rawg/{}.conf", name));

        fs::create_dir_all("~/.local/share/rawg")?;
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

    pub fn import_config(name: String) -> ServerConfig {
        let file_path = dirs::home_dir()
            .expect("Could not find home directory")
            .join(format!(".local/share/rawg/{}.conf", name));
        ServerConfig::new(name, file_path)
    }
}

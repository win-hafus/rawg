use crate::ConnectionStatus;
// use dirs;
use secrecy::{ExposeSecret, SecretString};
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
            ServerConfig::new(name, file_path, ConnectionStatus::Disconnected)
        }
    }
}

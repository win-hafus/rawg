use crate::header::ConnectionStatus;
use secrecy::{ExposeSecret, SecretString};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq)]
pub enum PrivilegeManager {
    Doas,
    Sudo,
}

impl PrivilegeManager {
    pub fn detect() -> Self {
        if Command::new("which")
            .arg("doas")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            PrivilegeManager::Doas
        } else {
            PrivilegeManager::Sudo
        }
    }

    pub fn needs_password(&self) -> bool {
        matches!(self, PrivilegeManager::Sudo)
    }
}

pub struct VpnManager;

impl VpnManager {
    pub fn connect(path: &PathBuf, privilege: &PrivilegeManager, password: Option<&SecretString>) -> ConnectionStatus {
        let path_str = path.to_str().expect("Not valid path!");
        match Self::run_awg_quick("up", path_str, privilege, password) {
            true => ConnectionStatus::Connected,
            false => ConnectionStatus::Disconnected,
        }
    }

    pub fn disconnect(path: &PathBuf, privilege: &PrivilegeManager, password: Option<&SecretString>) -> ConnectionStatus {
        let path_str = path.to_str().expect("Not valid path!");
        match Self::run_awg_quick("down", path_str, privilege, password) {
            true => ConnectionStatus::Disconnected,
            false => ConnectionStatus::Connected,
        }
    }

    fn run_awg_quick(cmd: &str, path_str: &str, privilege: &PrivilegeManager, password: Option<&SecretString>) -> bool {
        match privilege {
            PrivilegeManager::Doas => {
                Command::new("doas")
                    .args(["awg-quick", cmd, path_str])
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            }
            PrivilegeManager::Sudo => {
                let Some(password) = password else {
                    return false;
                };

                let Ok(mut child) = Command::new("sudo")
                    .args(["-S", "-p", "", "awg-quick", cmd, path_str])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                else {
                    return false;
                };

                if let Some(mut stdin) = child.stdin.take() {
                    if writeln!(stdin, "{}", password.expose_secret()).is_err() {
                        return false;
                    }
                }

                child
                    .wait_with_output()
                    .map(|out| out.status.success())
                    .unwrap_or(false)
            }
        }
    }

    /// Копирует конфиг в `~/.local/share/rawg/<name>.conf`.
    pub fn save_config(path: &PathBuf, name: &str) -> io::Result<()> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)?;

        let content = fs::read_to_string(path)?;
        let dest = config_dir.join(format!("{}.conf", name));
        fs::write(dest, content)?;

        Ok(())
    }

    /// Удаляет конфиг `~/.local/share/rawg/<name>.conf`.
    pub fn remove_config(name: &str) -> io::Result<()> {
        let file_path = Self::config_dir().join(format!("{}.conf", name));
        fs::remove_file(file_path)
    }

    /// Проверяет наличие обязательных секций WireGuard/AmneziaWG конфига.
    pub fn validate_config(path: &PathBuf) -> bool {
        let Ok(content) = fs::read_to_string(path) else {
            return false;
        };

        let required = [
            "[Interface]",
            "[Peer]",
            "PrivateKey",
            "Address",
            "PublicKey",
            "Endpoint",
        ];

        required.iter().all(|field| content.contains(field))
    }

    /// Проверяет статус подключения через `ip link show <name>`.
    pub fn check_connection_status(name: &str) -> ConnectionStatus {
        Command::new("ip")
            .args(["link", "show", name])
            .output()
            .map(|out| {
                if out.status.success() {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            })
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    /// Загружает все серверы из `~/.local/share/rawg/*.conf`.
    pub fn load_servers() -> Vec<crate::header::ServerConfig> {
        use crate::header::ServerConfig;

        let config_dir = Self::config_dir();

        let Ok(entries) = fs::read_dir(&config_dir) else {
            return vec![];
        };

        entries
            .flatten()
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "conf"))
            .map(|e| {
                let path = e.path();
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let status = Self::check_connection_status(&name);
                ServerConfig::new(name, path, status)
            })
            .collect()
    }

    fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".local/share/rawg")
    }
}

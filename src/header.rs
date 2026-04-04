use std::path::PathBuf;

pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

pub struct ServerConfig {
    pub name: String,
    pub path: PathBuf,
    pub status: ConnectionStatus,
}

impl ConnectionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Disconnected => "Disconnected",
        }
    }
}

impl ServerConfig {
    pub fn new(name: String, path: PathBuf, status: ConnectionStatus) -> Self {
        Self { name, path, status }
    }
}

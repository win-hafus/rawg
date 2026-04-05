use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}

pub struct ServerConfig {
    pub name: String,
    pub path: PathBuf,
    pub status: ConnectionStatus,
}

impl ServerConfig {
    pub fn new(name: String, path: PathBuf, status: ConnectionStatus) -> Self {
        Self { name, path, status }
    }
}
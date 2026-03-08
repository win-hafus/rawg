use std::{fs, io, path::PathBuf};
mod vpn_manager;
use vpn_manager::VpnManager;

pub enum ConnectionStatus {
    Connected(String),
    Disconnected,
}

pub struct ServerConfig {
    name: String,
    path: PathBuf,
}

impl ServerConfig {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

struct App {
    pub servers: Vec<String>,
    pub manager: VpnManager,
    pub status: ConnectionStatus,
    pub selected_index: usize,
    pub should_quit: bool,
}

impl App {
    fn handle_import_config(&self, name: String, path: &str) {}
}

fn main() {
    let mut app = App {
        servers: Vec::new(),
        manager: VpnManager,
        status: ConnectionStatus::Disconnected,
        selected_index: 0,
        should_quit: false,
    };
}

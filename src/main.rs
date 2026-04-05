mod file_explorer;
mod header;
mod ui;
mod vpn_manager;
mod widget;

use crate::{
    file_explorer::FileExplorer, header::ConnectionStatus, header::ServerConfig,
    ui::render_file_explorer, vpn_manager::VpnManager,
};

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::ListState};
use secrecy::SecretString;

struct App {
    pub servers: Vec<ServerConfig>,
    pub explorer: FileExplorer,
    pub list_state: ListState,
    sudo_password: Option<SecretString>,
    pub input_buffer: String,
    pub show_auth_popup: bool,
    pub show_explorer: bool,
    pub status_message: Option<String>,
    pub exit: bool,
}

fn load_servers() -> Vec<ServerConfig> {
    let config_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".local/share/rawg");

    let Ok(entries) = std::fs::read_dir(&config_dir) else {
        return vec![];
    };

    entries
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "conf")
                .unwrap_or(false)
        })
        .map(|e| {
            let path = e.path();
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let status = VpnManager::check_connection_status(&name); // ← проверяем статус
            ServerConfig::new(name, path, status)
        })
        .collect()
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // Ждём событие не дольше 2 секунд
            if event::poll(std::time::Duration::from_secs(2))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            } else {
                // Таймаут истёк — обновляем статусы
                self.refresh_statuses();
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        if self.show_explorer {
            render_file_explorer(frame, frame.area(), &self.explorer);
        } else {
            frame.render_widget(self, frame.area());
        }
    }
    fn refresh_statuses(&mut self) {
        for server in &mut self.servers {
            server.status = VpnManager::check_connection_status(&server.name);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.show_auth_popup {
            self.handle_popup_key_event(key_event);
        } else if self.show_explorer {
            self.handle_explorer_key_event(key_event);
        } else {
            self.handle_main_key_event(key_event);
        }
    }

    fn handle_main_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.status_message = None,
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.down(),
            KeyCode::Char('k') => self.up(),
            KeyCode::Char('a') => self.show_explorer = true,
            KeyCode::Char('d') => self.remove_selected_config(),
            KeyCode::Enter => self.connection(),
            _ => {}
        }
    }

    fn handle_popup_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                self.sudo_password = Some(secrecy::SecretString::from(self.input_buffer.clone()));
                self.input_buffer.clear();
                self.show_auth_popup = false;

                self.connection();
            }
            KeyCode::Esc => {
                self.show_auth_popup = false;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
    }
    fn handle_explorer_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.show_explorer = false,
            KeyCode::Char('j') => self.explorer.move_down(20),
            KeyCode::Char('k') => self.explorer.move_up(),
            KeyCode::Enter => {
                if let Some(conf_path) = self.explorer.enter() {
                    if !VpnManager::validate_config(&conf_path) {
                        self.status_message = Some("Invalid WireGuard config!".to_string());
                        self.show_explorer = false;
                        return;
                    }

                    let name = conf_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    match VpnManager::save_config(conf_path, name) {
                        Ok(_) => {
                            self.servers = load_servers();
                            self.show_explorer = false;
                            self.status_message = None;
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn up(&mut self) {
        self.status_message = None;
        self.list_state.select_previous();
    }

    fn down(&mut self) {
        self.status_message = None;
        self.list_state.select_next();
    }

    fn connection(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(server) = self.servers.get_mut(index) else {
            return;
        };

        if self.sudo_password.is_none() {
            self.show_auth_popup = true;
            return;
        }

        let password = self.sudo_password.as_ref().unwrap();

        server.status = match server.status {
            ConnectionStatus::Disconnected => VpnManager::connect(server.path.clone(), password),
            ConnectionStatus::Connected => VpnManager::disconnect(server.path.clone(), password),
        };
    }
    fn remove_selected_config(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(server) = self.servers.get(index) else {
            return;
        };

        match VpnManager::remove_config(server.name.clone()) {
            Ok(_) => {
                self.servers = load_servers();
                // Корректируем выделение если удалили последний элемент
                if self.servers.is_empty() {
                    self.list_state.select(None);
                } else if index >= self.servers.len() {
                    self.list_state.select(Some(self.servers.len() - 1));
                }
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut app = App {
        servers: load_servers(),
        explorer: FileExplorer::new(),
        list_state: ListState::default().with_selected(Some(0)),

        status_message: None,
        show_auth_popup: false,
        show_explorer: false,
        input_buffer: String::new(),
        sudo_password: None,
        exit: false,
    };

    ratatui::run(|terminal| app.run(terminal))
}

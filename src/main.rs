mod file_explorer;
mod header;
mod ui;
mod vpn_manager;
mod widget;

use crate::{
    file_explorer::FileExplorer,
    header::{ConnectionStatus, ServerConfig},
    ui::render_file_explorer,
    vpn_manager::{PrivilegeManager, VpnManager},
};

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::ListState};
use secrecy::SecretString;

struct App {
    pub servers: Vec<ServerConfig>,
    pub explorer: FileExplorer,
    pub list_state: ListState,
    pub privilege_manager: PrivilegeManager,
    sudo_password: Option<SecretString>,
    pub input_buffer: String,
    pub show_auth_popup: bool,
    pub show_explorer: bool,
    pub status_message: Option<String>,
    pub exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            servers: VpnManager::load_servers(),
            explorer: FileExplorer::new(),
            list_state: ListState::default().with_selected(Some(0)),
            privilege_manager: PrivilegeManager::detect(),
            status_message: None,
            show_auth_popup: false,
            show_explorer: false,
            input_buffer: String::new(),
            sudo_password: None,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // Ждём событие не дольше 2 секунд, затем обновляем статусы
            if event::poll(std::time::Duration::from_secs(2))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            } else {
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
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => self.nav_down(),
            KeyCode::Char('k') => self.nav_up(),
            KeyCode::Char('a') => self.show_explorer = true,
            KeyCode::Char('d') => self.remove_selected_config(),
            KeyCode::Enter => self.toggle_connection(),
            _ => {}
        }
    }

    fn handle_popup_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                self.sudo_password = Some(SecretString::from(self.input_buffer.clone()));
                self.input_buffer.clear();
                self.show_auth_popup = false;
                self.toggle_connection();
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
            KeyCode::Enter => self.explorer_select(),
            _ => {}
        }
    }

    fn explorer_select(&mut self) {
        let Some(conf_path) = self.explorer.enter() else {
            return;
        };

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

        match VpnManager::save_config(&conf_path, &name) {
            Ok(_) => {
                self.servers = VpnManager::load_servers();
                self.show_explorer = false;
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }

    fn nav_up(&mut self) {
        self.status_message = None;
        self.list_state.select_previous();
    }

    fn nav_down(&mut self) {
        self.status_message = None;
        self.list_state.select_next();
    }

    fn toggle_connection(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(server) = self.servers.get_mut(index) else {
            return;
        };

        // Для sudo нужен пароль — показываем попап если ещё не введён
        if self.privilege_manager.needs_password() && self.sudo_password.is_none() {
            self.show_auth_popup = true;
            return;
        }

        let password = self.sudo_password.as_ref();
        let prev_status = server.status.clone();

        server.status = match server.status {
            ConnectionStatus::Disconnected => VpnManager::connect(&server.path, &self.privilege_manager, password),
            ConnectionStatus::Connected => VpnManager::disconnect(&server.path, &self.privilege_manager, password),
        };

        // Если статус не изменился — команда не удалась
        if server.status == prev_status && self.privilege_manager == PrivilegeManager::Doas {
            self.status_message = Some(
                "doas failed. Add to /etc/doas.conf: permit nopass <user> as root cmd awg-quick"
                    .to_string(),
            );
        }
    }

    fn remove_selected_config(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(server) = self.servers.get(index) else {
            return;
        };

        match VpnManager::remove_config(&server.name) {
            Ok(_) => {
                self.servers = VpnManager::load_servers();
                // Корректируем выделение если удалили последний элемент
                let new_selected = if self.servers.is_empty() {
                    None
                } else {
                    Some(index.min(self.servers.len() - 1))
                };
                self.list_state.select(new_selected);
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut app = App::new();
    ratatui::run(|terminal| app.run(terminal))
}

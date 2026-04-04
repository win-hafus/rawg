mod header;
mod vpn_manager;
mod widget;

use crate::{header::ConnectionStatus, header::ServerConfig, vpn_manager::VpnManager};

use std::{io, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, widgets::ListState};
use secrecy::SecretString;

struct App {
    pub servers: Vec<ServerConfig>,
    pub list_state: ListState,
    sudo_password: Option<SecretString>,
    pub input_buffer: String,
    pub show_auth_popup: bool,
    pub exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        } else {
            self.handle_main_key_event(key_event);
        }
    }

    fn handle_main_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.down(),
            KeyCode::Char('k') => self.up(),
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

    fn exit(&mut self) {
        self.exit = true;
    }

    fn up(&mut self) {
        self.list_state.select_previous();
    }

    fn down(&mut self) {
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
}

fn main() -> io::Result<()> {
    let mut app = App {
        servers: vec![
            ServerConfig::new(
                String::from("Netherlands"),
                PathBuf::from("/home/hfv5/.local/share/rawg/nl0.conf"),
                ConnectionStatus::Disconnected,
            ),
            ServerConfig::new(
                String::from("USA"),
                PathBuf::from("/home/hfv5/.local/share/rawg/usa0.conf"),
                ConnectionStatus::Disconnected,
            ),
        ],
        list_state: ListState::default().with_selected(Some(0)),
        exit: false,

        show_auth_popup: false,
        input_buffer: String::new(),
        sudo_password: None,
    };

    ratatui::run(|terminal| app.run(terminal))
}

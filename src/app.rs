use std::error::Error;

use crate::terminal_session::TerminalSession;
use arboard::Clipboard;

pub struct App {
    pub terminals: Vec<TerminalSession>,
    pub current_tab_index: usize,
    pub username: String,
    pub insert_mode: bool,
    pub clipboard: Option<Clipboard>,
    pub editing_tab_index: Option<usize>,
    pub tab_rename_buffer: String,
}

impl App {
    pub fn new(username: String) -> Result<Self, Box<dyn Error>> {
        let mut app = App {
            terminals: Vec::new(),
            current_tab_index: 0,
            username,
            insert_mode: true, // Default to Insert mode (typing)
            clipboard: Clipboard::new().ok(),
            editing_tab_index: None,
            tab_rename_buffer: String::new(),
        };

        app.add_new_tab(None);
        Ok(app)
    }

    pub fn add_new_tab(&mut self, _dir: Option<String>) {
        if let Ok(session) = TerminalSession::new(self.terminals.len() + 1) {
            self.terminals.push(session);
            self.current_tab_index = self.terminals.len().saturating_sub(1);
        }
    }

    pub fn close_current_tab(&mut self) {
        if self.terminals.len() <= 1 {
            return;
        }

        self.terminals.remove(self.current_tab_index);
        if self.current_tab_index >= self.terminals.len() && !self.terminals.is_empty() {
            self.current_tab_index = self.terminals.len() - 1;
        }
    }

    pub fn next_tab(&mut self) {
        if self.terminals.is_empty() {
            return;
        }
        self.current_tab_index = (self.current_tab_index + 1) % self.terminals.len();
    }

    pub fn previous_tab(&mut self) {
        if self.terminals.is_empty() {
            return;
        }
        if self.current_tab_index == 0 {
            self.current_tab_index = self.terminals.len() - 1;
        } else {
            self.current_tab_index -= 1;
        }
    }

    pub fn get_current_terminal(&mut self) -> Option<&mut TerminalSession> {
        self.terminals.get_mut(self.current_tab_index)
    }
}

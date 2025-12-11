use chrono::Local;
use ratatui::style::Color;
use std::{
    env,
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone)]
pub struct OutputLine {
    pub text: String,
    pub color: Color,
}

pub struct TerminalSession {
    output: Arc<Mutex<Vec<OutputLine>>>,
    pub input_buffer: String,
    pub cursor_pos: usize,
    command_history: Vec<String>,
    history_index: Option<usize>,
    current_dir: String,
    pub title: String,

    // Selection & Scrolling
    pub scroll_offset: usize,                    // Lines from bottom
    pub visual_cursor_line: usize,               // Absolute line index in output
    pub selection_start: Option<(usize, usize)>, // (line_idx, char_idx)
    pub output_line_count: usize,                // Track total lines for efficient clamping
}

impl TerminalSession {
    pub fn new(tab_number: usize) -> Result<Self, Box<dyn std::error::Error>> {
        // Default to C:\ as requested
        let current_dir = r"C:\".to_string();
        // Set process current dir to match, so spawned commands start there
        env::set_current_dir(&current_dir).ok();

        Ok(TerminalSession {
            output: Arc::new(Mutex::new(vec![])),
            input_buffer: String::new(),
            cursor_pos: 0,
            command_history: Vec::new(),
            history_index: None,
            current_dir,
            title: format!("Tab {}", tab_number),
            scroll_offset: 0,
            visual_cursor_line: 0,
            selection_start: None,
            output_line_count: 0,
        })
    }

    pub fn execute_command(&mut self) {
        let cmd_text = self.input_buffer.trim().to_string();

        // Echo command to output with TIME
        let time = Local::now().format("%H:%M:%S"); // e.g., 14:30:05
        let prompt_line = format!("{} @{} -> {}", "Started", time, cmd_text);

        self.append_output(OutputLine {
            text: prompt_line,
            color: Color::Rgb(152, 195, 121), // Echo color (green-ish like prompt)
        });

        if cmd_text.is_empty() {
            self.input_buffer.clear();
            self.cursor_pos = 0;
            return;
        }

        // Reset scroll on new command
        self.scroll_offset = 0;
        self.selection_start = None;

        self.command_history.push(cmd_text.clone());
        self.history_index = None; // Reset history index

        // Handle built-in commands
        if cmd_text.starts_with("cd ") {
            let path = cmd_text.trim_start_matches("cd ").trim();
            match env::set_current_dir(path) {
                Ok(_) => {
                    if let Ok(new_dir) = env::current_dir() {
                        self.current_dir = new_dir.to_string_lossy().to_string();
                    }
                }
                Err(e) => {
                    self.append_output(OutputLine {
                        text: format!("Error: {}", e),
                        color: Color::Rgb(171, 178, 191),
                    });
                }
            }
        } else if cmd_text == "clear" || cmd_text == "cls" {
            self.clear_output();
        } else if cmd_text == "exit" {
            self.append_output(OutputLine {
                text: "Use Ctrl+Q to quit or Ctrl+W to close tab".to_string(),
                color: Color::Rgb(229, 192, 123),
            });
        } else if cmd_text == "export" {
            // Collect content first (requires lock)
            let file_content = {
                let output_guard = self.output.lock().unwrap();
                let mut content = String::new();
                content.push_str("=== Terminal Export ===\n");
                content.push_str(&format!("Date: {}\n", Local::now().to_string()));
                content.push_str("=======================\n\n");

                for line in output_guard.iter() {
                    content.push_str(&line.text);
                    content.push('\n');
                }
                content
            }; // Lock released here

            let output_clone = self.output.clone();

            thread::spawn(move || {
                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                let default_name = format!("terminal_export_{}.txt", timestamp);

                let file_path_opt = rfd::FileDialog::new()
                    .set_file_name(&default_name)
                    .add_filter("Text", &["txt"])
                    .save_file();

                let mut output = output_clone.lock().unwrap();

                if let Some(path) = file_path_opt {
                    match std::fs::write(&path, file_content) {
                        Ok(_) => {
                            output.push(OutputLine {
                                text: format!("Successfully exported to {:?}", path),
                                color: Color::Rgb(152, 195, 121),
                            });
                        }
                        Err(e) => {
                            output.push(OutputLine {
                                text: format!("Failed to export: {}", e),
                                color: Color::Rgb(224, 108, 117),
                            });
                        }
                    }
                } else {
                    output.push(OutputLine {
                        text: "Export cancelled.".to_string(),
                        color: Color::Rgb(229, 192, 123),
                    });
                }
            });
        } else {
            // Execute in PowerShell
            let output_clone = self.output.clone();
            let command = cmd_text.clone();

            thread::spawn(move || {
                match Command::new("powershell")
                    .arg("-NoLogo")
                    .arg("-NoProfile")
                    .arg("-Command")
                    .arg(&command)
                    .output()
                {
                    Ok(result) => {
                        let mut output = output_clone.lock().unwrap();

                        // stdout
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        for line in stdout.lines() {
                            output.push(OutputLine {
                                text: line.to_string(),
                                color: Color::Rgb(196, 214, 255),
                            });
                        }

                        // stderr
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        for line in stderr.lines() {
                            if !line.trim().is_empty() {
                                output.push(OutputLine {
                                    text: line.to_string(),
                                    color: Color::Rgb(224, 108, 117),
                                });
                            }
                        }

                        // exit code
                        if let Some(code) = result.status.code() {
                            if code != 0 {
                                output.push(OutputLine {
                                    text: format!("[Exit code: {}]", code),
                                    color: Color::Rgb(229, 192, 123),
                                });
                            }
                        }

                        // Keep last 1000 lines
                        let len = output.len();
                        if len > 1000 {
                            output.drain(0..len - 1000);
                        }
                    }
                    Err(e) => {
                        let mut output = output_clone.lock().unwrap();
                        output.push(OutputLine {
                            text: format!("Error: {}", e),
                            color: Color::Rgb(224, 108, 117),
                        });
                    }
                }
            });
        }

        self.input_buffer.clear();
        self.cursor_pos = 0;
    }

    fn append_output(&mut self, line: OutputLine) {
        let mut output = self.output.lock().unwrap();
        output.push(line);
        self.output_line_count = output.len();
        self.visual_cursor_line = self.output_line_count; // Move to end
    }

    pub fn clear_output(&mut self) {
        let mut output = self.output.lock().unwrap();
        output.clear();
        self.output_line_count = 0;
        self.scroll_offset = 0;
        self.selection_start = None;
        self.visual_cursor_line = 0;
    }

    pub fn handle_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_pos > 0 {
            self.input_buffer.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor_pos < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_pos);
        }
    }

    pub fn handle_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn handle_right(&mut self) {
        if self.cursor_pos < self.input_buffer.len() {
            self.cursor_pos += 1;
        }
    }

    // Normal Mode Navigation (Selection)
    pub fn move_visual_up(&mut self) {
        if self.visual_cursor_line > 0 {
            self.visual_cursor_line -= 1;
            // Adjust scroll if cursor moves above viewport
            // (Viewport logic usually handled in UI rendering, but we can hint scroll here)
            // For simplicity, let's keep visual_cursor logic simple here and implement viewport calc in main.
        }
    }

    pub fn move_visual_down(&mut self) {
        let max_lines = self.get_output_len();
        if self.visual_cursor_line < max_lines {
            self.visual_cursor_line += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        // Scroll up means looking at older output (increasing offset)
        // Max offset takes us to the top
        let max_len = self.get_output_len();
        if self.scroll_offset < max_len {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_down(&mut self) {
        // Scroll down means looking at newer output (decreasing offset)
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn start_selection(&mut self) {
        if self.selection_start.is_none() {
            self.selection_start = Some((self.visual_cursor_line, 0)); // Start at beginning of line for simplicity
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let start = self.selection_start?;
        let end = (self.visual_cursor_line, 0); // End at current visual cursor line

        let min = std::cmp::min(start.0, end.0);
        let max = std::cmp::max(start.0, end.0);

        let output = self.output.lock().unwrap();
        if min >= output.len() {
            return None;
        }

        let effective_max = std::cmp::min(max + 1, output.len());

        let mut text = String::new();
        for i in min..effective_max {
            if i < output.len() {
                text.push_str(&output[i].text);
                text.push('\n');
            }
        }
        Some(text)
    }

    pub fn handle_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.command_history.len() - 1);
                self.input_buffer = self.command_history[self.command_history.len() - 1].clone();
                self.cursor_pos = self.input_buffer.len();
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
                self.input_buffer = self.command_history[idx - 1].clone();
                self.cursor_pos = self.input_buffer.len();
            }
            _ => {}
        }
    }

    pub fn handle_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.command_history.len() - 1 {
                self.history_index = Some(idx + 1);
                self.input_buffer = self.command_history[idx + 1].clone();
                self.cursor_pos = self.input_buffer.len();
            } else {
                self.history_index = None;
                self.input_buffer.clear();
                self.cursor_pos = 0;
            }
        }
    }

    pub fn get_output(&self) -> Vec<OutputLine> {
        self.output.lock().unwrap().clone()
    }

    pub fn get_output_len(&self) -> usize {
        self.output.lock().unwrap().len()
    }

    pub fn get_current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn get_current_dir_name(&self) -> String {
        std::path::Path::new(&self.current_dir)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.current_dir)
            .to_string()
    }
}

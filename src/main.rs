mod app;
mod config;
mod terminal_session;

use app::App;
use config::Config;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Tabs},
};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get username
    let mut username = Config::load().username;
    if username == "user" {
        if let Some(new_username) = Config::prompt_for_username() {
            username = new_username.clone();
            Config::save_username(&new_username).ok();
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let mut app = App::new(username)?;
    let res = run_app(&mut terminal, &mut app);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Handle Tab Rename Input
                if let Some(idx) = app.editing_tab_index {
                    match key.code {
                        KeyCode::Enter => {
                            if !app.tab_rename_buffer.trim().is_empty() {
                                if let Some(term) = app.terminals.get_mut(idx) {
                                    term.title = app.tab_rename_buffer.clone();
                                }
                            }
                            app.editing_tab_index = None;
                            app.tab_rename_buffer.clear();
                        }
                        KeyCode::Esc => {
                            app.editing_tab_index = None;
                            app.tab_rename_buffer.clear();
                        }
                        KeyCode::Char(c) => {
                            app.tab_rename_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            app.tab_rename_buffer.pop();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Global Shortcuts
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.add_new_tab(None);
                        continue;
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.close_current_tab();
                        continue;
                    }
                    KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(term) = app.get_current_terminal() {
                            term.clear_output();
                        }
                        continue;
                    }
                    KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            app.previous_tab();
                        } else {
                            app.next_tab();
                        }
                        continue;
                    }
                    // Handle Shift+Tab (often sent as BackTab)
                    KeyCode::BackTab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.previous_tab();
                        continue;
                    }
                    _ => {}
                }

                // Mode-Specific Input
                if app.insert_mode {
                    // Paste (Ctrl+V)
                    if key.code == KeyCode::Char('v')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        let text = app.clipboard.as_mut().and_then(|cb| cb.get_text().ok());
                        if let Some(t) = text {
                            if let Some(term) = app.get_current_terminal() {
                                for c in t.chars() {
                                    term.handle_char(c);
                                }
                            }
                        }
                        continue;
                    }

                    // --- INSERT MODE ---
                    match key.code {
                        KeyCode::Esc => app.insert_mode = false,
                        KeyCode::Char(c) => {
                            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                if let Some(term) = app.get_current_terminal() {
                                    term.handle_char(c);
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_backspace();
                            }
                        }
                        KeyCode::Delete => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_delete();
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(term) = app.get_current_terminal() {
                                term.execute_command();
                            }
                        }
                        KeyCode::Left => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_left();
                            }
                        }
                        KeyCode::Right => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_right();
                            }
                        }
                        KeyCode::Up => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_up();
                            }
                        }
                        KeyCode::Down => {
                            if let Some(term) = app.get_current_terminal() {
                                term.handle_down();
                            }
                        }
                        KeyCode::Home => {
                            if let Some(term) = app.get_current_terminal() {
                                term.cursor_pos = 0;
                            }
                        }
                        KeyCode::End => {
                            if let Some(term) = app.get_current_terminal() {
                                term.cursor_pos = term.input_buffer.len();
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Copy (y or Ctrl+C)
                    let is_copy = (key.code == KeyCode::Char('y'))
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL));

                    if is_copy {
                        let text_to_copy = app
                            .get_current_terminal()
                            .and_then(|t| t.get_selected_text());
                        if let Some(text) = text_to_copy {
                            if let Some(cb) = &mut app.clipboard {
                                cb.set_text(text).ok();
                            }
                            if let Some(term) = app.get_current_terminal() {
                                term.clear_selection();
                            }
                        }
                        continue;
                    }

                    // --- NORMAL MODE (Visual/Scroll) ---
                    if let Some(term) = app.get_current_terminal() {
                        match key.code {
                            KeyCode::Char('i') => app.insert_mode = true,
                            KeyCode::Esc => term.clear_selection(),

                            // Visual Selection Toggle
                            KeyCode::Char('v') => term.start_selection(),

                            // Navigation (Scroll/Move Visual Cursor)
                            KeyCode::Up => term.move_visual_up(),
                            KeyCode::Down => term.move_visual_down(),
                            KeyCode::Char('k') => term.move_visual_up(),
                            KeyCode::Char('j') => term.move_visual_down(),
                            _ => {}
                        }
                    }
                }
            } else if let Event::Mouse(mouse) = event::read()? {
                if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    // Check valid click in Tabs area
                    // We assume tabs are at the top, row 0-2 (height 3)
                    // The tabs are inside a Block with Borders::ALL, so content is at row 1?
                    // Tabs widget renders titles with separator.

                    if mouse.row >= 1 && mouse.row <= 3 {
                        // Very rough approximation of tab widths
                        let mut x_offset = 1; // Left border... maybe
                        let mut clicked_tab = None;

                        for (i, term) in app.terminals.iter().enumerate() {
                            let title = &term.title;
                            let width = title.len() as u16 + 2; // + padding spaces

                            // Check collision
                            if mouse.column >= x_offset && mouse.column < x_offset + width {
                                clicked_tab = Some((i, term.title.clone()));
                                break;
                            }

                            x_offset += width + 1; // + separator
                        }

                        if let Some((i, title)) = clicked_tab {
                            app.current_tab_index = i;
                            app.editing_tab_index = Some(i);
                            app.tab_rename_buffer = title;
                        }
                    }
                } else if mouse.kind == MouseEventKind::ScrollDown {
                    if let Some(term) = app.get_current_terminal() {
                        term.scroll_down();
                    }
                } else if mouse.kind == MouseEventKind::ScrollUp {
                    if let Some(term) = app.get_current_terminal() {
                        term.scroll_up();
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Terminal
            Constraint::Length(1), // Mode Indicator
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    draw_terminal(f, app, chunks[1]);
    draw_mode_indicator(f, app, chunks[2]);
}

fn draw_mode_indicator(f: &mut Frame, app: &App, area: Rect) {
    let (text, color) = if app.insert_mode {
        (" -- INSERT -- ", Color::Rgb(152, 195, 121)) // Green
    } else {
        (" -- NORMAL -- ", Color::Rgb(229, 192, 123)) // Yellow
    };

    let paragraph = Paragraph::new(Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(paragraph, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let bg = Color::Rgb(24, 26, 32);
    let primary = Color::Rgb(97, 175, 239);
    let highlight = Color::Rgb(198, 120, 221);

    let titles: Vec<String> = app
        .terminals
        .iter()
        .enumerate()
        .map(|(i, term)| {
            if Some(i) == app.editing_tab_index {
                format!(" {}_ ", app.tab_rename_buffer)
            } else {
                format!(" {} ", term.title)
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(bg).fg(primary)),
        )
        .select(app.current_tab_index)
        .style(Style::default().fg(primary).bg(bg))
        .highlight_style(
            Style::default()
                .fg(highlight)
                .bg(Color::Rgb(41, 46, 66))
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn draw_terminal(f: &mut Frame, app: &mut App, area: Rect) {
    let bg = Color::Rgb(24, 26, 32);
    let primary = Color::Rgb(97, 175, 239);
    let secondary = Color::Rgb(198, 120, 221);
    let accent = Color::Rgb(152, 195, 121);
    let cursor_color = Color::Rgb(229, 192, 123);

    let username = app.username.clone();
    let insert_mode = app.insert_mode; // Copy bool value before borrow

    if let Some(term) = app.get_current_terminal() {
        let output = term.get_output();
        let height = (area.height as usize).saturating_sub(2);

        let mut lines = Vec::new();

        // Render Output with Visual Selection Highlight logic
        // For simplicity, we just iterate valid output lines
        // Improve viewport logic: scroll upwards from bottom or use scroll_offset

        // Very basic view logic: currently just showing last N lines
        // To implement scrolling correctly, we should slice output based on scroll position.
        // But for now, to ensure basic functionality + Prompt, we stick to "tail" view unless in Visual mode?
        // Let's implement basic scrolling:

        let total_lines = output.len();
        // Determine start index for view
        // If we want to strictly follow visual_cursor for view in Normal mode, it's complex.
        // Let's stick to "Prompt View" (tail) for Insert Mode, and maybe allow scrolling in Normal.
        // Simplest: just map all lines, and Ratatui Paragraph handles simple wrapping, OR we slice.

        // We will stick to slicing for performance and simplicity in prompt placement
        let view_start = total_lines.saturating_sub(height + term.scroll_offset);
        let view_end = total_lines.saturating_sub(term.scroll_offset);

        for (i, ol) in output.iter().enumerate().take(view_end).skip(view_start) {
            let mut style = Style::default().fg(ol.color).bg(bg);

            // Simple line highlight if selected (entire line)
            if let Some((start_line, _)) = term.selection_start {
                let min = std::cmp::min(start_line, term.visual_cursor_line);
                let max = std::cmp::max(start_line, term.visual_cursor_line);
                if i >= min && i <= max {
                    style = style.bg(Color::Rgb(62, 68, 82)); // Selection highlight
                }
            } else if !insert_mode && i == term.visual_cursor_line {
                style = style.bg(Color::Rgb(40, 44, 52)); // Current visual cursor line
            }

            lines.push(Line::from(Span::styled(&ol.text, style)));
        }

        // Add prompt line (Only if at bottom or always? Always append for now)
        if insert_mode {
            let mut prompt_spans = vec![
                Span::styled(
                    format!("{}@", username),
                    Style::default()
                        .fg(primary)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", term.get_current_dir_name()),
                    Style::default()
                        .fg(secondary)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "-> ",
                    Style::default()
                        .fg(accent)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &term.input_buffer[..term.cursor_pos],
                    Style::default().bg(bg).fg(Color::Rgb(224, 230, 235)),
                ),
                Span::styled("█", Style::default().fg(cursor_color).bg(bg)),
            ];
            if term.cursor_pos < term.input_buffer.len() {
                prompt_spans.push(Span::styled(
                    &term.input_buffer[term.cursor_pos..],
                    Style::default().bg(bg).fg(Color::Rgb(224, 230, 235)),
                ));
            }
            lines.push(Line::from(prompt_spans));
        }

        let terminal_widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .padding(Padding::left(1))
                    .style(Style::default().bg(bg).fg(primary)),
            )
            .style(Style::default().bg(bg).fg(Color::Rgb(224, 230, 235)));

        f.render_widget(terminal_widget, area);
    }
}

use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use std::{fs, path::PathBuf};
use tui_textarea::{Input, Key};

use crate::themes::Theme;

#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub name: String,
    pub path: PathBuf,
    pub is_parent: bool,
}

pub struct SavePopup {
    items: Vec<DirectoryItem>,
    list_state: ListState,
    focused: bool,
    current_path: PathBuf,
    filename: String,
}

impl SavePopup {
    pub fn new(_theme: &'static Theme) -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        let mut popup = Self {
            items: Vec::new(),
            list_state: ListState::default(),
            focused: false,
            current_path: current_path.clone(),
            filename: String::new(),
        };
        popup.load_directories(&current_path);
        popup.list_state.select(Some(0));
        popup
    }

    fn load_directories(&mut self, path: &PathBuf) {
        self.items.clear();

        // Add parent directory option if not at root
        if let Some(parent) = path.parent() {
            self.items.push(DirectoryItem {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_parent: true,
            });
        }

        // Load subdirectories
        if let Ok(entries) = fs::read_dir(path) {
            let mut dirs = Vec::new();

            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden directories
                if name.starts_with('.') {
                    continue;
                }

                if entry_path.is_dir() {
                    dirs.push(DirectoryItem {
                        name,
                        path: entry_path,
                        is_parent: false,
                    });
                }
            }

            // Sort directories alphabetically
            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            self.items.extend(dirs);
        }

        // Reset selection to first item
        if !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.filename = filename.to_string();
    }

    pub fn get_save_path(&self) -> Option<PathBuf> {
        if self.filename.is_empty() {
            return None;
        }
        
        let mut path = self.current_path.clone();
        path.push(&self.filename);
        Some(path)
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> (bool, bool) {
        match key_event.into() {
            Input { key: Key::Esc, .. } => (true, false),
            Input { key: Key::Enter, .. } => {
                // Save file in current directory
                if !self.filename.is_empty() {
                    (true, true)
                } else {
                    (false, false)
                }
            }
            input => {
                // Handle directory navigation
                match input.key {
                    Key::Up | Key::Char('k') => {
                        let selected = self.list_state.selected().unwrap_or(0);
                        if selected > 0 {
                            self.list_state.select(Some(selected - 1));
                        }
                        (false, false)
                    }
                    Key::Down | Key::Char('j') => {
                        let selected = self.list_state.selected().unwrap_or(0);
                        if selected < self.items.len().saturating_sub(1) {
                            self.list_state.select(Some(selected + 1));
                        }
                        (false, false)
                    }
                    Key::Char('h') => {
                        // Navigate to parent directory
                        if let Some(parent) = self.current_path.parent() {
                            let new_path = parent.to_path_buf();
                            self.current_path = new_path.clone();
                            self.load_directories(&new_path);
                        }
                        (false, false)
                    }
                    Key::Char('l') => {
                        // Navigate into selected directory
                        if let Some(selected) = self.list_state.selected() {
                            if selected < self.items.len() {
                                let item = &self.items[selected];
                                let new_path = item.path.clone();
                                self.current_path = new_path.clone();
                                self.load_directories(&new_path);
                            }
                        }
                        (false, false)
                    }
                    _ => (false, false)
                }
            }
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn clear(&mut self) {
        self.filename.clear();
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(theme.background))
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        // Create layout: current path, directory list, help
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Current path
                Constraint::Min(1),    // Directory list
                Constraint::Length(1), // Help text
            ])
            .split(inner);

        // Render current path
        let path_display = format!(" Save to: {}", self.current_path.display());
        let path_paragraph = Paragraph::new(path_display)
            .style(Style::default().fg(theme.foreground).bg(theme.background));
        path_paragraph.render(layout[0], buf);

        // Render directory list
        let items: Vec<ListItem> = self.items.iter().map(|item| {
            let display_name = if item.is_parent {
                format!(" {}/", item.name)
            } else {
                format!(" {}/", item.name)
            };
            
            ListItem::new(display_name)
        }).collect();

        let list_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.border));

        let list = List::new(items)
            .block(list_block)
            .style(Style::default().fg(theme.foreground))
            .highlight_style(Style::default().bg(theme.accent).fg(theme.background));

        let mut list_state = self.list_state.clone();
        ratatui::widgets::StatefulWidget::render(list, layout[1], buf, &mut list_state);

        // Render help text
        let help_text = Line::from(vec![
            Span::styled("↑↓/jk", Style::default().fg(theme.accent)),
            Span::styled(" navigate, ", Style::default().fg(theme.foreground)),
            Span::styled("Enter/l", Style::default().fg(theme.accent)),
            Span::styled(" open, ", Style::default().fg(theme.foreground)),
            Span::styled("h", Style::default().fg(theme.accent)),
            Span::styled(" back, ", Style::default().fg(theme.foreground)),
            Span::styled("Enter", Style::default().fg(theme.accent)),
            Span::styled(" save here, ", Style::default().fg(theme.foreground)),
            Span::styled("Esc", Style::default().fg(theme.accent)),
            Span::styled(" cancel", Style::default().fg(theme.foreground)),
        ]);

        let help_paragraph = Paragraph::new(help_text).style(Style::default().bg(theme.background));
        help_paragraph.render(layout[2], buf);
    }
}
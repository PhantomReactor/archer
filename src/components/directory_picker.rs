use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use std::{fs, path::PathBuf};

use crate::themes::Theme;

#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub name: String,
    pub path: PathBuf,
    pub is_parent: bool,
}

pub struct DirectoryPicker {
    items: Vec<DirectoryItem>,
    state: ListState,
    focused: bool,
    current_path: PathBuf,
    selected_path: Option<PathBuf>,
}

impl DirectoryPicker {
    pub fn new(_theme: &'static Theme) -> Self {
        let current_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut picker = Self {
            items: Vec::new(),
            state: ListState::default(),
            focused: false,
            current_path: current_path.clone(),
            selected_path: None,
        };
        picker.load_directories(&current_path);
        picker.state.select(Some(0));
        picker
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

        // Add current directory option
        self.items.push(DirectoryItem {
            name: ".".to_string(),
            path: path.clone(),
            is_parent: false,
        });

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
            self.state.select(Some(0));
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = self.state.selected().unwrap_or(0);
                if selected > 0 {
                    self.state.select(Some(selected - 1));
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = self.state.selected().unwrap_or(0);
                if selected < self.items.len().saturating_sub(1) {
                    self.state.select(Some(selected + 1));
                }
                true
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(selected) = self.state.selected() {
                    if selected < self.items.len() {
                        let item = &self.items[selected];
                        if item.name == "." {
                            // Select current directory
                            self.selected_path = Some(self.current_path.clone());
                            return true;
                        } else {
                            // Navigate to selected directory
                            let new_path = item.path.clone();
                            self.current_path = new_path.clone();
                            self.load_directories(&new_path);
                        }
                    }
                }
                true
            }
            KeyCode::Char('h') => {
                // Navigate to parent directory
                if let Some(parent) = self.current_path.parent() {
                    let new_path = parent.to_path_buf();
                    self.current_path = new_path.clone();
                    self.load_directories(&new_path);
                }
                true
            }
            KeyCode::Char('s') => {
                // Select current directory with 's' key
                self.selected_path = Some(self.current_path.clone());
                true
            }
            _ => false,
        }
    }

    pub fn get_selected_path(&mut self) -> Option<PathBuf> {
        self.selected_path.take()
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
        self.selected_path = None;
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        // Clear the area
        Clear.render(area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(theme.background))
            .border_style(Style::default().fg(theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        let help_height = 1;
        let gap_height = 1;
        let list_height = inner.height.saturating_sub(help_height + gap_height + 1);

        // Directory list area
        let list_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: list_height,
        };

        // Help area
        let help_area = Rect {
            x: inner.x,
            y: inner.y + list_height + gap_height,
            width: inner.width,
            height: help_height,
        };

        // Render directory list
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let display_name = if item.name == "." {
                    format!(" {} (select current)", item.name)
                } else if item.is_parent {
                    format!(" {}/", item.name)
                } else {
                    format!(" {}/", item.name)
                };

                ListItem::new(display_name)
            })
            .collect();

        let list_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.border));

        let list = List::new(items)
            .block(list_block)
            .style(Style::default().fg(theme.foreground))
            .highlight_style(Style::default().bg(theme.cursor).fg(theme.background));

        ratatui::widgets::StatefulWidget::render(list, list_area, buf, &mut self.state);

        // Render help text with styled spans (matching search popup style)
        let help_text = Line::from(vec![
            Span::styled("Enter/L", Style::default().fg(theme.accent)),
            Span::styled(" to open/select, ", Style::default().fg(theme.foreground)),
            Span::styled("H", Style::default().fg(theme.accent)),
            Span::styled(" to go back, ", Style::default().fg(theme.foreground)),
            Span::styled("S", Style::default().fg(theme.accent)),
            Span::styled(
                " to select current, ",
                Style::default().fg(theme.foreground),
            ),
            Span::styled("↑↓/jk", Style::default().fg(theme.accent)),
            Span::styled(" to navigate, ", Style::default().fg(theme.foreground)),
            Span::styled("Esc", Style::default().fg(theme.accent)),
            Span::styled(" to cancel", Style::default().fg(theme.foreground)),
        ]);

        let help_paragraph = Paragraph::new(help_text).style(Style::default().bg(theme.background));
        help_paragraph.render(help_area, buf);
    }
}

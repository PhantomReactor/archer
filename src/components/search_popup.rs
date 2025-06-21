use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use std::process::Command;
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::themes::Theme;

#[derive(Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub match_type: MatchType,
    pub preview: Option<String>,
}

#[derive(Clone)]
pub enum MatchType {
    FileName,
    Content { line_number: usize },
}

pub struct SearchPopup {
    textarea: TextArea<'static>,
    focused: bool,
    theme: &'static Theme,
    search_results: Vec<SearchResult>,
    list_state: ListState,
    selected_index: usize,
}

impl SearchPopup {
    pub fn new(theme: &'static Theme) -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_style(Style::default().fg(theme.foreground));
        textarea.set_cursor_style(Style::default().fg(theme.foreground));
        textarea.set_placeholder_text("Search for files...");
        let block = Block::default().style(Style::default().fg(theme.border));
        textarea.set_block(block);
        Self {
            textarea,
            focused: false,
            theme,
            search_results: Vec::new(),
            list_state: ListState::default(),
            selected_index: 0,
        }
    }

    pub fn focus(&mut self) {
        self.textarea
            .set_cursor_style(Style::default().bg(self.theme.cursor));
        self.textarea.move_cursor(CursorMove::End);
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.textarea
            .set_cursor_style(Style::default().fg(self.theme.foreground));
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn clear(&mut self) {
        self.textarea = TextArea::new(vec![]);
        self.textarea
            .set_cursor_line_style(Style::default().fg(self.theme.foreground));
        self.textarea
            .set_cursor_style(Style::default().fg(self.theme.foreground));
        self.textarea.set_placeholder_text("Search for files...");
        self.search_results.clear();
        self.selected_index = 0;
        self.list_state.select(None);
    }

    pub fn get_input(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn get_selected_file(&self) -> Option<String> {
        if self.selected_index < self.search_results.len() {
            Some(self.search_results[self.selected_index].file_path.clone())
        } else {
            None
        }
    }

    pub fn search_files(&mut self, query: &str) {
        self.search_results.clear();
        self.selected_index = 0;

        let mut all_results = Vec::new();

        if query.trim().is_empty() {
            // Show all files when search is empty
            let file_output = Command::new("rg")
                .args(&["--files"])
                .current_dir(".")
                .output();

            if let Ok(output) = file_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.is_empty() {
                        all_results.push(SearchResult {
                            file_path: line.to_string(),
                            match_type: MatchType::FileName,
                            preview: None,
                        });
                    }
                }
            }
        } else {
            // First, search for files by name
            let file_output = Command::new("rg")
                .args(&["--files", "--glob", &format!("*{}*", query)])
                .current_dir(".")
                .output();

            if let Ok(output) = file_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.is_empty() {
                        all_results.push(SearchResult {
                            file_path: line.to_string(),
                            match_type: MatchType::FileName,
                            preview: None,
                        });
                    }
                }
            }

            // Then, search for content within files
            let content_output = Command::new("rg")
                .args(&[
                    "--line-number",
                    "--no-heading",
                    "--max-count",
                    "1", // Only show first match per file
                    "--context",
                    "0",
                    query,
                ])
                .current_dir(".")
                .output();

            if let Ok(output) = content_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.is_empty() {
                        // Parse ripgrep output: file:line:content
                        let parts: Vec<&str> = line.splitn(3, ':').collect();
                        if parts.len() >= 3 {
                            let file_path = parts[0];
                            let line_num_str = parts[1];
                            let content = parts[2];

                            if let Ok(line_number) = line_num_str.parse::<usize>() {
                                // Check if we already have this file from filename search
                                let already_exists =
                                    all_results.iter().any(|r| r.file_path == file_path);

                                if !already_exists {
                                    all_results.push(SearchResult {
                                        file_path: file_path.to_string(),
                                        match_type: MatchType::Content { line_number },
                                        preview: Some(content.trim().to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Sort results: filename matches first, then content matches
            all_results.sort_by(|a, b| match (&a.match_type, &b.match_type) {
                (MatchType::FileName, MatchType::Content { .. }) => std::cmp::Ordering::Less,
                (MatchType::Content { .. }, MatchType::FileName) => std::cmp::Ordering::Greater,
                _ => a.file_path.cmp(&b.file_path),
            });
        }

        self.search_results = all_results.into_iter().take(50).collect();

        if !self.search_results.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> (bool, bool) {
        // Handle navigation keys first (including Ctrl combinations)
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                if !self.search_results.is_empty() {
                    self.selected_index =
                        (self.selected_index + 1).min(self.search_results.len() - 1);
                    self.list_state.select(Some(self.selected_index));
                }
                return (false, false);
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                if !self.search_results.is_empty() && self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.list_state.select(Some(self.selected_index));
                }
                return (false, false);
            }
            (KeyCode::Down, _) => {
                if !self.search_results.is_empty() {
                    self.selected_index =
                        (self.selected_index + 1).min(self.search_results.len() - 1);
                    self.list_state.select(Some(self.selected_index));
                }
                return (false, false);
            }
            (KeyCode::Up, _) => {
                if !self.search_results.is_empty() && self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.list_state.select(Some(self.selected_index));
                }
                return (false, false);
            }
            _ => {}
        }

        // Handle other keys through Input system
        match key_event.into() {
            Input { key: Key::Esc, .. } => (true, false),
            Input {
                key: Key::Enter, ..
            } => {
                if !self.search_results.is_empty()
                    && self.selected_index < self.search_results.len()
                {
                    (true, true)
                } else {
                    // If no results selected, perform search
                    let query = self.get_input();
                    self.search_files(&query);
                    (false, false)
                }
            }
            input => {
                self.textarea.input(input);
                // Auto-search as user types
                let query = self.get_input();
                self.search_files(&query);
                (false, false)
            }
        }
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

        // Create layout: search input at top, results below, help at bottom
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Search input
                Constraint::Min(1),    // Results
                Constraint::Length(1), // Help text
            ])
            .split(inner);

        // Add padding to search input area
        let search_input_area = Rect {
            x: layout[0].x + 1, // Left padding
            y: layout[0].y,
            width: layout[0].width.saturating_sub(1), // Adjust width for left padding
            height: layout[0].height,
        };

        // Render search input
        let mut textarea = self.textarea.clone();
        if self.focused {
            textarea.set_cursor_style(Style::default().bg(theme.cursor));
        } else {
            textarea.set_cursor_style(Style::default().fg(theme.foreground));
        }
        textarea.render(search_input_area, buf);

        // Add padding to results area
        let results_area = Rect {
            x: layout[1].x + 1, // Left padding
            y: layout[1].y,
            width: layout[1].width.saturating_sub(1), // Adjust width for left padding
            height: layout[1].height.saturating_sub(1), // Bottom padding
        };

        // Always render the results area with border
        //test
        let results_list = if !self.search_results.is_empty() {
            let items: Vec<ListItem> = self
                .search_results
                .iter()
                .enumerate()
                .map(|(i, result)| {
                    let style = if i == self.selected_index {
                        Style::default().bg(theme.accent).fg(theme.background)
                    } else {
                        Style::default().fg(theme.foreground)
                    };

                    // Create the display text based on match type (no icons)
                    let display_text = match &result.match_type {
                        MatchType::FileName => result.file_path.clone(),
                        MatchType::Content { line_number } => {
                            if let Some(preview) = &result.preview {
                                format!("{} (line {}): {}", result.file_path, line_number, preview)
                            } else {
                                format!("{} (line {})", result.file_path, line_number)
                            }
                        }
                    };

                    ListItem::new(display_text).style(style)
                })
                .collect();

            List::new(items)
        } else {
            List::new(vec![ListItem::new("No files found")])
        };

        let results_with_block = results_list
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            )
            .style(Style::default().bg(theme.background));

        let mut list_state = self.list_state.clone();
        ratatui::widgets::StatefulWidget::render(
            results_with_block,
            results_area,
            buf,
            &mut list_state,
        );

        // Render help text
        let help_text = if !self.search_results.is_empty() {
            Line::from(vec![
                Span::styled("↑↓/Ctrl+k/j", Style::default().fg(theme.accent)),
                Span::styled(" navigate, ", Style::default().fg(theme.foreground)),
                Span::styled("Enter", Style::default().fg(theme.accent)),
                Span::styled(" select, ", Style::default().fg(theme.foreground)),
                Span::styled("Esc", Style::default().fg(theme.accent)),
                Span::styled(" close", Style::default().fg(theme.foreground)),
            ])
        } else {
            Line::from(vec![
                Span::styled("Type", Style::default().fg(theme.accent)),
                Span::styled(
                    " to search files & content, ",
                    Style::default().fg(theme.foreground),
                ),
                Span::styled("Esc", Style::default().fg(theme.accent)),
                Span::styled(" to close", Style::default().fg(theme.foreground)),
            ])
        };

        let help_paragraph = Paragraph::new(help_text).style(Style::default().bg(theme.background));
        help_paragraph.render(layout[2], buf);
    }
}

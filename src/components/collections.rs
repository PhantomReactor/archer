use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
    style::{Style, Styled},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, StatefulWidget},
};
use std::{fs, path::PathBuf};

use crate::themes::Theme;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub depth: usize,
}

#[derive(Debug)]
pub struct Collections {
    items: Vec<FileItem>,
    state: ListState,
    focused: bool,
}

impl Collections {
    pub fn new(_theme: &Theme) -> Self {
        let root_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut collections = Self {
            items: Vec::new(),
            state: ListState::default(),
            focused: true,
        };
        collections.load_directory(&root_path, 0);
        collections.state.select(Some(0));
        collections
    }

    fn load_directory(&mut self, path: &PathBuf, depth: usize) {
        if let Ok(entries) = fs::read_dir(path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let item = FileItem {
                    name,
                    path: path.clone(),
                    is_dir,
                    expanded: false,
                    depth,
                };

                if is_dir {
                    dirs.push(item);
                } else {
                    files.push(item);
                }
            }

            dirs.sort_by(|a, b| a.name.cmp(&b.name));
            files.sort_by(|a, b| a.name.cmp(&b.name));

            if depth == 0 {
                self.items.clear();
            }

            self.items.extend(dirs);
            self.items.extend(files);
        }
    }

    fn get_display_items(&self, theme: &Theme) -> Vec<ListItem<'static>> {
        self.build_display_tree(&self.items, theme)
    }

    fn build_display_tree(&self, items: &[FileItem], theme: &Theme) -> Vec<ListItem<'static>> {
        let mut display_items = Vec::new();

        for (index, item) in items.iter().enumerate() {
            let mut spans = Vec::new();

            if item.depth > 0 {
                for d in 0..item.depth {
                    if d == item.depth - 1 {
                        let is_last = !items.iter().skip(index + 1).any(|other| {
                            other.depth == item.depth && other.path.parent() == item.path.parent()
                        });

                        if is_last {
                            spans.push(Span::styled(
                                "└─".to_string(),
                                Style::default().fg(theme.tree_lines),
                            ));
                        } else {
                            spans.push(Span::styled(
                                "│ ".to_string(),
                                Style::default().fg(theme.tree_lines),
                            ));
                        }
                    } else {
                        let has_more_at_level = items
                            .iter()
                            .skip(index + 1)
                            .any(|other| other.depth <= d + 1);

                        if has_more_at_level {
                            spans.push(Span::styled(
                                "│ ".to_string(),
                                Style::default().fg(theme.tree_lines),
                            ));
                        } else {
                            spans.push(Span::raw("  ".to_string()));
                        }
                    }
                }
            }

            let icon = if item.is_dir {
                if item.expanded {
                    "📂 "
                } else {
                    "📁 "
                }
            } else {
                "📄 "
            };
            spans.push(Span::raw(icon));
            spans.push(Span::raw(item.name.clone()));

            display_items.push(ListItem::new(Line::from(spans)));
        }

        display_items
    }

    pub fn next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn expand_folder(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected < self.items.len()
                && self.items[selected].is_dir
                && !self.items[selected].expanded
            {
                self.items[selected].expanded = true;
                let path = self.items[selected].path.clone();
                let depth = self.items[selected].depth + 1;

                if let Ok(entries) = fs::read_dir(&path) {
                    let mut new_items = Vec::new();
                    let mut dirs = Vec::new();
                    let mut files = Vec::new();

                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();

                        if name.starts_with('.') {
                            continue;
                        }

                        let is_dir = entry_path.is_dir();
                        let item = FileItem {
                            name,
                            path: entry_path,
                            is_dir,
                            expanded: false,
                            depth,
                        };

                        if is_dir {
                            dirs.push(item);
                        } else {
                            files.push(item);
                        }
                    }

                    dirs.sort_by(|a, b| a.name.cmp(&b.name));
                    files.sort_by(|a, b| a.name.cmp(&b.name));
                    new_items.extend(dirs);
                    new_items.extend(files);

                    for (i, item) in new_items.into_iter().enumerate() {
                        self.items.insert(selected + 1 + i, item);
                    }
                }
            }
        }
    }

    pub fn collapse_folder(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected < self.items.len() {
                let is_dir = self.items[selected].is_dir;
                let expanded = self.items[selected].expanded;
                let depth = self.items[selected].depth;

                if is_dir && expanded {
                    self.items[selected].expanded = false;

                    let i = selected + 1;
                    while i < self.items.len() && self.items[i].depth > depth {
                        self.items.remove(i);
                    }
                } else if depth > 0 {
                    // Navigate to parent folder and collapse it
                    let target_depth = depth - 1;
                    for i in (0..selected).rev() {
                        if self.items[i].depth == target_depth && self.items[i].is_dir {
                            self.state.select(Some(i));
                            self.collapse_folder();
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_keys(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('j') => {
                    self.next();
                    true
                }
                KeyCode::Char('k') => {
                    self.previous();
                    true
                }
                KeyCode::Char('l') => {
                    self.expand_folder();
                    true
                }
                KeyCode::Char('h') => {
                    self.collapse_folder();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let display_items = self.get_display_items(theme);
        let list = List::new(display_items)
            .set_style(Style::default().bg(theme.background))
            .highlight_style(Style::default().bg(theme.selection));

        StatefulWidget::render(list, area, buf, &mut self.state);
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
}

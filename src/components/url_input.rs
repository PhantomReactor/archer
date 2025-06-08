use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::themes::Theme;

#[derive(Debug)]
pub struct UrlInput {
    textarea: TextArea<'static>,
    focused: bool,
    theme: &'static Theme,
}

impl UrlInput {
    pub fn new(theme: &'static Theme) -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_cursor_line_style(Style::default().fg(theme.foreground));
        textarea.set_cursor_style(Style::default().fg(theme.foreground));
        textarea.set_placeholder_text("Enter URL here");
        let block = Block::bordered()
            .style(Style::default().fg(theme.border))
            .border_type(BorderType::Rounded);
        textarea.set_block(block);
        Self {
            textarea,
            focused: false,
            theme,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.into() {
            Input {
                key: Key::Char('m'),
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => false,
            input => {
                self.textarea.input(input);
                true
            }
        }
    }

    pub fn get_input(self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.textarea.render(area, buf);
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
}

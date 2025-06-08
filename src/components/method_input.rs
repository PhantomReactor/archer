use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(Debug)]
pub struct MethodInput {
    textarea: TextArea<'static>,
    focused: bool,
}

impl MethodInput {
    pub fn new() -> Self {
        let mut textarea = TextArea::new(vec!["GET".to_string()]);
        textarea.set_cursor_line_style(Style::default().fg(Color::White));
        textarea.set_cursor_style(Style::default().fg(Color::White));
        // textarea.set_style(Style::default().bg(Color::Rgb(93, 93, 93)));
        let block = Block::bordered()
            .style(Style::default().fg(Color::Rgb(93, 93, 93)))
            .border_type(BorderType::Rounded);
        textarea.set_block(block);
        Self {
            textarea,
            focused: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.into() {
            // Ignore newline inputs to keep it single-line
            Input {
                key: Key::Char('m'),
                ctrl: true,
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

    pub fn get_method(self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.textarea.render(area, buf);
    }

    pub fn focus(&mut self) {
        self.textarea
            .set_cursor_style(Style::default().bg(Color::Rgb(250, 178, 255)));
        self.textarea.move_cursor(CursorMove::End);
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.textarea
            .set_cursor_style(Style::default().fg(Color::White));
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
}

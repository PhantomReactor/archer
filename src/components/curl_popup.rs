use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::themes::Theme;

pub struct CurlPopup {
    textarea: TextArea<'static>,
    focused: bool,
    theme: &'static Theme,
}

impl CurlPopup {
    pub fn new(theme: &'static Theme) -> Self {
        let mut textarea = TextArea::new(vec![]);
        textarea.set_style(Style::default().fg(theme.foreground));
        textarea.set_cursor_style(Style::default().fg(theme.foreground));
        textarea.set_placeholder_text("Paste your cURL command here");
        let block = Block::default().style(Style::default().fg(theme.border));
        textarea.set_block(block);
        Self {
            textarea,
            focused: false,
            theme,
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
        self.textarea
            .set_placeholder_text("Paste your cURL command here");
    }

    pub fn get_input(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> bool {
        match key_event.into() {
            Input { key: Key::Esc, .. } => true,
            Input {
                key: Key::Enter,
                alt: true,
                ..
            } => true,
            input => {
                self.textarea.input(input);
                false
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

        let help_height = 1;
        let textarea_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(help_height + 1),
        };
        let help_area = Rect {
            x: inner.x,
            y: inner.y + textarea_area.height + 1,
            width: inner.width,
            height: help_height,
        };

        let mut textarea = self.textarea.clone();
        if self.focused {
            textarea.set_cursor_style(Style::default().bg(theme.cursor));
        } else {
            textarea.set_cursor_style(Style::default().fg(theme.foreground));
        }

        textarea.render(textarea_area, buf);

        let help_text = Line::from(vec![
            Span::styled("Alt+Enter", Style::default().fg(theme.accent)),
            Span::styled(" to import, ", Style::default().fg(theme.foreground)),
            Span::styled("Esc", Style::default().fg(theme.accent)),
            Span::styled(" to close", Style::default().fg(theme.foreground)),
        ]);

        let help_paragraph = Paragraph::new(help_text).style(Style::default().bg(theme.background));

        help_paragraph.render(help_area, buf);
    }
}
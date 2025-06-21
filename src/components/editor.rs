use edtui::{
    EditorEventHandler, EditorState, EditorStatusLine, EditorTheme, EditorView, SyntaxHighlighter, Lines,
};
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::{buffer::Buffer, layout::Rect, style::Style};

use ratatui::crossterm::event::KeyEvent;

use crate::themes::Theme;

pub struct Editor {
    editor_state: EditorState,
    event_handler: EditorEventHandler,
    focused: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            editor_state: EditorState::default(),
            event_handler: EditorEventHandler::default(),
            focused: false,
        }
    }
}

impl Editor {
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) {
        self.event_handler.on_event(
            crossterm::event::Event::Key(key_event),
            &mut self.editor_state,
        );
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &'static Theme) {
        let theme_name = "ayu-dark";
        let extension = "json";
        let syntax_highlighter = SyntaxHighlighter::new(theme_name, extension);
        let editor_theme = EditorTheme::default()
            .base(Style::default().bg(theme.background))
            .status_line(
                EditorStatusLine::default()
                    .style_text(Style::default().bg(theme.background))
                    .style_line(Style::default().bg(theme.background)),
            );
        let mut_theme: EditorTheme;
        if !self.focused {
            mut_theme = editor_theme.hide_cursor();
        } else {
            mut_theme =
                editor_theme.cursor_style(Style::default().fg(Color::Black).bg(theme.cursor));
        }
        EditorView::new(&mut self.editor_state)
            .theme(mut_theme)
            .syntax_highlighter(Some(syntax_highlighter))
            .render(area, buf);
    }

    pub fn get_text(&self) -> String {
        self.editor_state.lines.iter_row()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn set_text(&mut self, text: String) {
        self.editor_state.lines = Lines::from(text.as_str());
    }

    pub fn clear(&mut self) {
        self.editor_state.lines = Lines::from("");
    }
}

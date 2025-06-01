use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled},
    widgets::{Block, BorderType, List, ListState, StatefulWidget},
};

#[derive(Debug)]
pub struct Collections {
    items: List<'static>,
    focused: bool,
}

impl Collections {
    pub fn new() -> Self {
        let list = List::new(vec!["test".to_string()]);
        let mut_list = list.set_style(Style::default().bg(Color::Rgb(38, 38, 38)));
        let block = Block::default().border_type(BorderType::Rounded);
        let styled_list = mut_list.block(block);
        Self {
            items: styled_list,
            focused: true,
        }
    }

    pub fn handle_keys(self) {}

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default();
        StatefulWidget::render(&self.items, area, buf, &mut state);
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

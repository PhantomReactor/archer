use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut explorer = 0;
        let mut response = 0;
        if self.show_explorer {
            explorer = 20;
        }
        if self.show_response {
            response = 20;
        }
        let constraints = [
            Constraint::Percentage(explorer),
            Constraint::Percentage(100 - explorer - response),
            Constraint::Percentage(response),
        ];

        let app_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);
        let request_layout = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(app_layout[1]);
        let method_url = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(1)])
            .split(request_layout[0]);
        let paragraph = Paragraph::default()
            .fg(Color::Cyan)
            .bg(Color::Rgb(33, 33, 33))
            .centered();

        paragraph.render(area, buf);
        self.method_input.render(method_url[0], buf);
        self.url_input.render(method_url[1], buf);
        self.request_input.render(request_layout[1], buf);
        if self.show_explorer {
            self.collections.render(app_layout[0], buf);
        }
    }
}

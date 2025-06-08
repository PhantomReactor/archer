use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
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
            .horizontal_margin(1)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(app_layout[1]);

        let method_url = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(10), Constraint::Min(1)])
            .split(request_layout[0]);

        let l = Layout::default()
            .vertical_margin(1)
            .constraints([Constraint::Length(2), Constraint::Fill(1)])
            .split(request_layout[1]);
        let options_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(6),
                Constraint::Length(2),
                Constraint::Length(7),
                Constraint::Length(2),
                Constraint::Length(4),
            ])
            .split(l[0]);
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(l[1]);
        let paragraph = Paragraph::default().bg(Color::Rgb(33, 33, 33)).centered();

        let block = Block::bordered()
            .style(Style::default().fg(Color::Rgb(93, 93, 93)))
            .border_type(BorderType::Rounded);

        paragraph.render(area, buf);
        self.method_input.render(method_url[0], buf);
        self.url_input.render(method_url[1], buf);
        Paragraph::default()
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Rgb(93, 93, 93))),
            )
            .render(l[0], buf);
        Paragraph::default()
            .block(block)
            .render(request_layout[1], buf);
        for i in 1..6 {
            if i % 2 == 0 {
                continue;
            }
            let mut req_type: String = "".to_string();
            if i == 1 {
                req_type = "Params".to_string();
            } else if i == 3 {
                req_type = "Headers".to_string();
            } else if i == 5 {
                req_type = "Body".to_string();
            }

            if ((i + 1) / 2) - 1 == self.current_focus {
                Paragraph::new(req_type)
                    .fg(Color::Rgb(250, 178, 131))
                    .block(
                        Block::default()
                            .borders(Borders::BOTTOM)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(93, 93, 93))),
                    )
                    .render(options_layout[i], buf);
            } else {
                Paragraph::new(req_type)
                    .fg(Color::White)
                    .block(
                        Block::default()
                            .borders(Borders::BOTTOM)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(93, 93, 93))),
                    )
                    .render(options_layout[i], buf);
            }
        }
        self.request_input[self.current_focus].render(body_layout[0], buf);
        if self.show_explorer {
            self.collections.render(app_layout[0], buf);
        }
    }
}

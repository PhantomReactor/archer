use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};
use std::rc::Rc;

use crate::app::App;
use crate::themes::Theme;

fn create_app_layout(area: Rect, show_explorer: bool, show_response: bool) -> Rc<[Rect]> {
    let mut explorer = 0;
    let mut response = 0;
    if show_explorer {
        explorer = 20;
    }
    if show_response {
        response = (0.5 * ((100 - explorer) as f64)) as i32;
    }
    let constraints = [
        Constraint::Percentage(explorer),
        Constraint::Percentage(100 - explorer - response as u16),
        Constraint::Percentage(response as u16),
    ];

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area)
}

fn create_request_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .horizontal_margin(1)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(area)
}

fn create_method_url_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(1)])
        .split(area)
}

fn create_content_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .vertical_margin(1)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(area)
}

fn create_options_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(7),
            Constraint::Length(2),
            Constraint::Length(4),
        ])
        .split(area)
}

fn create_body_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Percentage(100)])
        .split(area)
}

fn create_response_layout(area: Rect) -> Rc<[Rect]> {
    let res = Layout::default()
        .vertical_margin(1)
        .constraints([Constraint::Fill(1)])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Fill(1)])
        .split(res[0])
}

fn render_options(
    options_layout: &Rc<[Rect]>,
    current_focus: usize,
    buf: &mut Buffer,
    theme: &Theme,
) {
    for i in 1..6 {
        if i % 2 == 0 {
            continue;
        }
        let req_type = match i {
            1 => "Params",
            3 => "Headers",
            5 => "Body",
            _ => "",
        };

        let color = if ((i + 1) / 2) - 1 == current_focus {
            theme.accent
        } else {
            theme.foreground
        };

        Paragraph::new(req_type)
            .fg(color)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            )
            .render(options_layout[i], buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = self.theme;
        let app_layout = create_app_layout(area, self.show_explorer, self.show_response);
        let request_layout = create_request_layout(app_layout[1]);
        let method_url = create_method_url_layout(request_layout[0]);
        let content_layout = create_content_layout(request_layout[1]);
        let options_layout = create_options_layout(content_layout[0]);
        let body_layout = create_body_layout(content_layout[1]);
        let response_layout = create_response_layout(app_layout[2]);
        let paragraph = Paragraph::default().bg(theme.background).centered();

        paragraph.render(area, buf);
        self.method_input.render(method_url[0], buf);
        self.url_input.render(method_url[1], buf);
        Paragraph::default()
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.border)),
            )
            .render(content_layout[0], buf);
        Paragraph::default()
            .block(
                Block::bordered()
                    .style(Style::default().fg(theme.border))
                    .border_type(BorderType::Rounded),
            )
            .render(request_layout[1], buf);
        render_options(&options_layout, self.current_focus, buf, theme);
        self.editors[self.current_focus].render(body_layout[0], buf, self.theme);
        if self.show_explorer {
            self.collections.render(app_layout[0], buf);
        }
        if self.show_response {
            Paragraph::default()
                .block(
                    Block::bordered()
                        .style(Style::default().fg(theme.border))
                        .border_type(BorderType::Rounded),
                )
                .render(app_layout[2], buf);
            self.response.render(response_layout[0], buf, theme);
        }

        if self.show_popup {
            let block = Block::bordered().title("Popup");
            let area = popup_area(area, 60, 20);
            Clear.render(area, buf);
            block.render(area, buf);
        }
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

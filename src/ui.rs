use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, StatefulWidget},
};
use ratatui_image::StatefulImage;
use std::rc::Rc;

use crate::app::App;
use crate::themes::Theme;

fn create_app_layout(area: Rect, show_explorer: bool, show_response: bool) -> Rc<[Rect]> {
    let mut explorer = 0;
    let mut response = 0;
    if show_explorer {
        explorer = 18;
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
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
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
    Layout::default()
        .vertical_margin(1)
        .horizontal_margin(1)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(area)
}

fn create_response_options_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(7),
            Constraint::Length(2),
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .split(area)
}

fn create_response_body_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(2)
        .constraints([Constraint::Percentage(100)])
        .split(area)
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

fn render_response_options(
    options_layout: &Rc<[Rect]>,
    current_focus: usize,
    buf: &mut Buffer,
    theme: &Theme,
    response_status: &Option<String>,
    response_time: &Option<u128>,
    is_loading: bool,
    spinner_state: usize,
) {
    for i in 1..6 {
        if i % 2 == 0 {
            continue;
        }
        let resp_type = match i {
            1 => "Response",
            3 => "Headers",
            5 => "Cookies",
            _ => "",
        };

        let color = if ((i + 1) / 2) - 1 == current_focus {
            theme.accent
        } else {
            theme.foreground
        };

        Paragraph::new(resp_type)
            .fg(color)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme.border)),
            )
            .render(options_layout[i], buf);
    }

    if options_layout.len() > 6 {
        let mut status_text = String::new();

        if is_loading {
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            status_text.push_str(spinner_chars[spinner_state]);
        } else {
            if let Some(status) = response_status {
                status_text.push_str(status);
            }
            if let Some(time) = response_time {
                if !status_text.is_empty() {
                    status_text.push_str("  ");
                }
                status_text.push_str(&format!("{}ms", time));
            }
        }

        if !status_text.is_empty() {
            let text_color = if is_loading {
                theme.accent
            } else {
                theme.foreground
            };
            Paragraph::new(status_text)
                .fg(text_color)
                .alignment(Alignment::Right)
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(theme.border)),
                )
                .render(options_layout[6], buf);
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = self.theme;
        let app_layout = create_app_layout(area, self.show_explorer, self.show_response);
        let request_layout = create_request_layout(app_layout[1]);
        let method_url = create_method_url_layout(request_layout[1]);
        let content_layout = create_content_layout(request_layout[2]);
        let options_layout = create_options_layout(content_layout[0]);
        let body_layout = create_body_layout(content_layout[1]);
        let response_layout = create_response_layout(app_layout[2]);
        let response_options_layout = create_response_options_layout(response_layout[0]);
        let response_body_layout = create_response_body_layout(response_layout[1]);
        let paragraph = Paragraph::default().bg(theme.background).centered();

        paragraph.render(area, buf);

        let display_name = if self.is_renaming {
            &self.rename_input
        } else {
            &self.request_name
        };

        let name_style = if self.is_renaming {
            Style::default().bg(theme.accent).fg(theme.background)
        } else {
            Style::default().bg(theme.accent).fg(theme.background)
        };

        let min_width = 20u16;
        let text_width = display_name.len() as u16;
        let actual_width = text_width.max(min_width);
        let area_width = request_layout[0].width;

        let start_x = if actual_width < area_width {
            request_layout[0].x + (area_width - actual_width) / 2
        } else {
            request_layout[0].x
        };

        let name_area = Rect {
            x: start_x,
            y: request_layout[0].y,
            width: actual_width.min(area_width),
            height: request_layout[0].height,
        };

        Paragraph::new(display_name.as_str())
            .style(name_style)
            .alignment(Alignment::Center)
            .render(name_area, buf);

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
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(theme.border)),
            )
            .render(request_layout[2], buf);
        render_options(&options_layout, self.current_focus, buf, theme);
        self.editors[self.current_focus].render(body_layout[0], buf, self.theme);
        if self.show_explorer {
            self.collections.render(app_layout[0], buf, theme);
        }
        if self.show_response {
            Paragraph::default()
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(theme.border)),
                )
                .render(app_layout[2], buf);
            Paragraph::default()
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(theme.border)),
                )
                .render(response_layout[0], buf);
            render_response_options(
                &response_options_layout,
                self.current_response_focus,
                buf,
                theme,
                &self.response_status,
                &self.response_time,
                self.is_loading,
                self.spinner_state,
            );
            match self.current_response_focus {
                0 => {
                    if self.response_is_image && self.image_display_enabled {
                        // Try to create image protocol if needed (should be fast now)
                        let _ = self.update_image_area(response_body_layout[0]);
                        
                        // Now render the image if we have one ready
                        if let Some(ref mut image_protocol) = self.response_image {
                            let image = StatefulImage::default();
                            image.render(response_body_layout[0], buf, image_protocol);
                        } else {
                            // Image not ready yet - just show text response
                            self.response.render(response_body_layout[0], buf, theme);
                        }
                    } else {
                        self.response.render(response_body_layout[0], buf, theme);
                    }
                }
                1 => self
                    .response_headers
                    .render(response_body_layout[0], buf, theme),
                2 => self
                    .response_cookies
                    .render(response_body_layout[0], buf, theme),
                _ => {
                    if self.response_is_image && self.image_display_enabled {
                        // Try to create image protocol if needed (should be fast now)
                        let _ = self.update_image_area(response_body_layout[0]);
                        
                        // Now render the image if we have one
                        if let Some(ref mut image_protocol) = self.response_image {
                            let image = StatefulImage::default();
                            image.render(response_body_layout[0], buf, image_protocol);
                        } else {
                            self.response.render(response_body_layout[0], buf, theme);
                        }
                    } else {
                        self.response.render(response_body_layout[0], buf, theme);
                    }
                }
            }
        }

        if self.show_popup {
            let block = Block::bordered().title("Popup");
            let area = popup_area(area, 60, 20);
            Clear.render(area, buf);
            block.render(area, buf);
        }

        if self.show_curl_popup {
            let popup_area = popup_area(area, 80, 40);
            self.curl_popup.render(popup_area, buf, theme);
        }

        if self.show_search_popup {
            let popup_area = popup_area(area, 60, 50);
            self.search_popup.render(popup_area, buf, theme);
        }

        if self.show_save_popup {
            let popup_area = popup_area(area, 60, 50);
            self.save_popup.render(popup_area, buf, theme);
        }

        if let Some(error) = &self.save_error {
            let error_area = popup_area(area, 60, 20);
            Clear.render(error_area, buf);
            let error_block = Block::bordered()
                .title("Error")
                .border_style(Style::default().fg(theme.accent));
            let inner = error_block.inner(error_area);
            error_block.render(error_area, buf);
            Paragraph::new(error.as_str())
                .style(Style::default().fg(theme.foreground))
                .render(inner, buf);
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

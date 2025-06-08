use std::usize;

use crate::components::collections::Collections;
use crate::components::method_input::MethodInput;
use crate::components::request_input::{Mode, Transition, Vim};
use crate::components::url_input::UrlInput;
use crate::event::{AppEvent, Event, EventHandler};
use crate::themes::{Theme, ARCHER};
use color_eyre::eyre::Ok;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    DefaultTerminal,
};

#[derive(Debug)]
pub struct App {
    pub show_explorer: bool,
    pub show_response: bool,
    pub running: bool,
    pub counter: u8,
    pub events: EventHandler,
    pub method_input: MethodInput,
    pub url_input: UrlInput,
    pub request_input: Vec<Vim>,
    pub collections: Collections,
    pub current_focus: usize,
    pub theme: &'static Theme,
}

impl Default for App {
    fn default() -> Self {
        let theme = &ARCHER;
        let mut request: Vec<Vim> = Vec::new();
        for _ in 0..5 {
            request.push(Vim::new(Mode::Normal, theme));
        }
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            method_input: MethodInput::new(theme),
            url_input: UrlInput::new(theme),
            request_input: request,
            collections: Collections::new(theme),
            show_explorer: false,
            show_response: false,
            current_focus: 0,
            theme,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            KeyCode::Char('r') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.current_focus = 2;
                if !self.request_input[self.current_focus].is_focused() {
                    self.unfocus();
                    self.request_input[self.current_focus].focus();
                }
                return Ok(());
            }
            KeyCode::Char('p') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.current_focus = 0;
                if !self.request_input[self.current_focus].is_focused() {
                    self.unfocus();
                    self.request_input[self.current_focus].focus();
                }
                return Ok(());
            }
            KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.current_focus = 1;
                if !self.request_input[self.current_focus].is_focused() {
                    self.unfocus();
                    self.request_input[self.current_focus].focus();
                }
                return Ok(());
            }
            KeyCode::Char('g') if key_event.modifiers == KeyModifiers::CONTROL => {
                if !self.method_input.is_focused() {
                    self.unfocus();
                    self.method_input.focus();
                }
                return Ok(());
            }
            KeyCode::Char('u') if key_event.modifiers == KeyModifiers::CONTROL => {
                if !self.url_input.is_focused() {
                    self.unfocus();
                    self.url_input.focus();
                }
                return Ok(());
            }

            KeyCode::Char('e') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.show_explorer = !self.show_explorer;
                if !self.collections.is_focused() {
                    self.unfocus();
                    self.collections.focus();
                }
                return Ok(());
            }
            _ => {
                if self.method_input.is_focused() {
                    self.method_input.handle_key(key_event);
                } else if self.url_input.is_focused() {
                    self.url_input.handle_key(key_event);
                } else {
                    let input = key_event.into();
                    let mut vim_clone = self.request_input[self.current_focus].clone();
                    let transition = vim_clone.transition(input);
                    match transition {
                        Transition::Mode(mode) if vim_clone.mode != mode => {
                            let mut new_vim = Vim::new(mode, self.theme);
                            new_vim.textarea = vim_clone.textarea;

                            self.request_input[self.current_focus] = new_vim;
                        }
                        Transition::Nop | Transition::Mode(_) => {
                            self.request_input[self.current_focus] = vim_clone;
                        }
                        Transition::Pending(input) => {
                            self.request_input[self.current_focus] = vim_clone.with_pending(input);
                        }
                        Transition::Quit => {
                            self.request_input[self.current_focus].unfocus();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    pub fn unfocus(&mut self) {
        self.request_input[self.current_focus].unfocus();
        self.url_input.unfocus();
        self.method_input.unfocus();
        self.collections.unfocus();
    }
}

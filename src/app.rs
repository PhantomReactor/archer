use std::collections::HashMap;
use std::result::Result::Ok;
use std::time::Instant;
use std::usize;

#[derive(Debug, Clone)]
pub struct ParsedRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: String,
    pub content_type: String,
}

use crate::components::collections::Collections;
use crate::components::curl_popup::CurlPopup;
use crate::components::directory_picker::DirectoryPicker;
use crate::components::editor::Editor;
use crate::components::method_input::MethodInput;
use crate::components::save_popup::SavePopup;
use crate::components::search_popup::SearchPopup;
use crate::components::url_input::UrlInput;
use crate::event::{AppEvent, Event, EventHandler, HttpResponseData};
use crate::themes::{ARCHER, Theme};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

pub struct App {
    pub show_explorer: bool,
    pub show_response: bool,
    pub running: bool,
    pub counter: u8,
    pub events: EventHandler,
    pub method_input: MethodInput,
    pub url_input: UrlInput,
    pub collections: Collections,
    pub current_focus: usize,
    pub theme: &'static Theme,
    pub show_popup: bool,
    pub curl_popup: CurlPopup,
    pub show_curl_popup: bool,
    pub search_popup: SearchPopup,
    pub show_search_popup: bool,
    pub save_popup: SavePopup,
    pub show_save_popup: bool,
    pub directory_picker: DirectoryPicker,
    pub show_directory_picker: bool,
    pub editors: Vec<Editor>,
    pub response: Editor,
    pub response_headers: Editor,
    pub response_cookies: Editor,
    pub current_response_focus: usize,
    pub response_status: Option<String>,
    pub response_time: Option<u128>,
    pub is_loading: bool,
    pub spinner_state: usize,
    pub spinner_counter: usize,
    pub request_name: String,
    pub is_renaming: bool,
    pub rename_input: String,
    pub save_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let theme = &ARCHER;
        let mut editors: Vec<Editor> = Vec::new();
        for _ in 0..5 {
            editors.push(Editor::default());
        }
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            method_input: MethodInput::new(theme),
            url_input: UrlInput::new(theme),
            collections: Collections::new(theme),
            show_explorer: false,
            show_response: false,
            current_focus: 0,
            theme,
            show_popup: false,
            curl_popup: CurlPopup::new(theme),
            show_curl_popup: false,
            search_popup: SearchPopup::new(theme),
            show_search_popup: false,
            save_popup: SavePopup::new(theme),
            show_save_popup: false,
            directory_picker: DirectoryPicker::new(theme),
            show_directory_picker: false,
            editors,
            response: Editor::default(),
            response_headers: Editor::default(),
            response_cookies: Editor::default(),
            current_response_focus: 0,
            response_status: None,
            response_time: None,
            is_loading: false,
            spinner_state: 0,
            spinner_counter: 0,
            request_name: "Untitled-Request".to_string(),
            is_renaming: false,
            rename_input: String::new(),
            save_error: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event).await?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                    AppEvent::HttpResponse(response_data) => {
                        self.handle_http_response(response_data)
                    }
                },
            }
        }
        Ok(())
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.save_error.is_some() {
            if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Enter {
                self.save_error = None;
                return Ok(());
            }
        }

        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            KeyCode::Char('r') if key_event.modifiers == KeyModifiers::ALT => {
                if !self.is_renaming {
                    self.is_renaming = true;
                    self.rename_input = self.request_name.clone();
                }
                return Ok(());
            }
            KeyCode::Enter if self.is_renaming => {
                self.save_renamed_request().await?;
                return Ok(());
            }
            KeyCode::Esc if self.is_renaming => {
                self.is_renaming = false;
                self.rename_input.clear();
                return Ok(());
            }
            KeyCode::Char('r') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.show_response = !self.show_response;
                if self.show_response {
                    self.unfocus();
                    self.response.focus();
                }
                return Ok(());
            }
            KeyCode::Char('b') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.current_focus = 2;
                if !self.editors[self.current_focus].is_focused() {
                    self.unfocus();
                    self.editors[self.current_focus].focus();
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
            KeyCode::Char('o') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.show_curl_popup = !self.show_curl_popup;
                if self.show_curl_popup {
                    self.unfocus();
                    self.curl_popup.focus();
                }
                return Ok(());
            }
            KeyCode::Char('f') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.show_search_popup = !self.show_search_popup;
                if self.show_search_popup {
                    self.unfocus();
                    self.search_popup.focus();
                    self.search_popup.search_files("");
                }
                return Ok(());
            }
            KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.show_save_popup = !self.show_save_popup;
                if self.show_save_popup {
                    self.unfocus();
                    self.save_popup.focus();
                    let filename = if self.request_name == "Untitled-Request" {
                        format!("{}.curl", uuid::Uuid::new_v4())
                    } else {
                        format!("{}.curl", self.request_name)
                    };
                    self.save_popup.set_filename(&filename);
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
            KeyCode::Char('h') if key_event.modifiers == KeyModifiers::CONTROL => {
                let currently_in_response = self.response.is_focused()
                    || self.response_headers.is_focused()
                    || self.response_cookies.is_focused();
                let currently_in_request = self.url_input.is_focused()
                    || self.method_input.is_focused()
                    || self.editors.iter().any(|e| e.is_focused());
                let currently_in_explorer = self.collections.is_focused();

                if currently_in_response {
                    self.unfocus();
                    self.url_input.focus();
                } else if currently_in_request && self.show_explorer {
                    self.unfocus();
                    self.collections.focus();
                } else if self.show_explorer && !currently_in_explorer {
                    self.unfocus();
                    self.collections.focus();
                }
                return Ok(());
            }
            KeyCode::Char('l') if key_event.modifiers == KeyModifiers::CONTROL => {
                let currently_in_response = self.response.is_focused()
                    || self.response_headers.is_focused()
                    || self.response_cookies.is_focused();
                let currently_in_request = self.url_input.is_focused()
                    || self.method_input.is_focused()
                    || self.editors.iter().any(|e| e.is_focused());
                let currently_in_explorer = self.collections.is_focused();

                if currently_in_explorer {
                    self.unfocus();
                    self.url_input.focus();
                } else if currently_in_request && self.show_response {
                    self.unfocus();
                    self.current_response_focus = 0;
                    self.response.focus();
                } else if self.show_response && !currently_in_response {
                    self.unfocus();
                    self.current_response_focus = 0;
                    self.response.focus();
                }
                return Ok(());
            }
            KeyCode::Char('n') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.new_request();
                return Ok(());
            }
            KeyCode::Char('t') if key_event.modifiers == KeyModifiers::ALT => {
                self.start_http_request().await?;
                return Ok(());
            }
            KeyCode::Char('j') if key_event.modifiers == KeyModifiers::CONTROL => {
                if self.method_input.is_focused() || self.url_input.is_focused() {
                    self.unfocus();
                    self.editors[self.current_focus].focus();
                }
                return Ok(());
            }
            KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => {
                if self.editors.iter().any(|e| e.is_focused()) {
                    self.unfocus();
                    self.url_input.focus();
                }
                return Ok(());
            }
            KeyCode::Tab => {
                if self.method_input.is_focused() {
                    self.unfocus();
                    self.url_input.focus();
                } else if self.url_input.is_focused() {
                    self.unfocus();
                    self.method_input.focus();
                } else if self.editors.iter().any(|e| e.is_focused()) {
                    self.current_focus = (self.current_focus + 1) % 3;
                    self.unfocus();
                    self.editors[self.current_focus].focus();
                } else if self.response.is_focused()
                    || self.response_headers.is_focused()
                    || self.response_cookies.is_focused()
                {
                    self.current_response_focus = (self.current_response_focus + 1) % 3;
                    self.unfocus();
                    match self.current_response_focus {
                        0 => self.response.focus(),
                        1 => self.response_headers.focus(),
                        2 => self.response_cookies.focus(),
                        _ => self.response.focus(),
                    }
                }
                return Ok(());
            }
            _ => {
                if self.is_renaming {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            self.rename_input.push(c);
                        }
                        KeyCode::Backspace => {
                            self.rename_input.pop();
                        }
                        _ => {}
                    }
                } else if self.method_input.is_focused() {
                    self.method_input.handle_key(key_event);
                } else if self.url_input.is_focused() {
                    self.url_input.handle_key(key_event);
                } else if self.collections.is_focused() {
                    let event = ratatui::crossterm::event::Event::Key(key_event);
                    if key_event.code == KeyCode::Enter {
                        if let Some(selected_file) = self.collections.get_selected_file() {
                            if !selected_file.is_dir && selected_file.name.ends_with(".curl") {
                                let file_path = selected_file.path.clone();
                                if let Err(e) = self.load_request_from_file(&file_path) {
                                    self.save_error = Some(format!("Failed to load file: {}", e));
                                }
                            }
                        }
                    } else {
                        self.collections.handle_keys(&event);
                    }
                } else if self.response.is_focused() {
                    if key_event.code != KeyCode::Char('i') {
                        self.response.handle_key(key_event);
                    }
                } else if self.response_headers.is_focused() {
                    if key_event.code != KeyCode::Char('i') {
                        self.response_headers.handle_key(key_event);
                    }
                } else if self.response_cookies.is_focused() {
                    if key_event.code != KeyCode::Char('i') {
                        self.response_cookies.handle_key(key_event);
                    }
                } else if self.show_curl_popup && self.curl_popup.is_focused() {
                    let should_close = self.curl_popup.handle_key(key_event);
                    if should_close {
                        if key_event.code == KeyCode::Enter
                            && key_event.modifiers == KeyModifiers::ALT
                        {
                            let curl_input = self.curl_popup.get_input();
                            self.parse_curl_command(&curl_input);
                        }
                        self.show_curl_popup = false;
                        self.curl_popup.unfocus();
                        self.curl_popup.clear();
                    }
                } else if self.show_search_popup && self.search_popup.is_focused() {
                    let (should_close, should_search) = self.search_popup.handle_key(key_event);
                    if should_close {
                        if should_search {
                            if let Some(selected_file) = self.search_popup.get_selected_file() {
                                if selected_file.ends_with(".curl") {
                                    let file_path = std::path::Path::new(&selected_file);
                                    if let Err(e) = self.load_request_from_file(file_path) {
                                        self.save_error =
                                            Some(format!("Failed to load file: {}", e));
                                    }
                                }
                            }
                        }
                        self.show_search_popup = false;
                        self.search_popup.unfocus();
                        self.search_popup.clear();
                    }
                } else if self.show_save_popup && self.save_popup.is_focused() {
                    let (should_close, should_save) = self.save_popup.handle_key(key_event);
                    if should_close {
                        if should_save {
                            if let Some(save_path) = self.save_popup.get_save_path() {
                                if let Err(e) = self.save_request_to_file(&save_path).await {
                                    self.save_error = Some(format!("Failed to save file: {}", e));
                                } else {
                                    if let Some(file_stem) = save_path.file_stem() {
                                        if let Some(name_str) = file_stem.to_str() {
                                            self.request_name = name_str.to_string();
                                        }
                                    }
                                    self.collections.refresh();
                                }
                            }
                        }
                        self.show_save_popup = false;
                        self.save_popup.unfocus();
                        self.save_popup.clear();
                    }
                } else {
                    self.editors[self.current_focus].handle_key(key_event);
                }
            }
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        if self.is_loading {
            self.spinner_counter += 1;
            if self.spinner_counter >= 3 {
                self.spinner_state = (self.spinner_state + 1) % 10;
                self.spinner_counter = 0;
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    pub fn new_request(&mut self) {
        self.url_input.clear();
        self.method_input.set_text("GET".to_string());

        for editor in &mut self.editors {
            editor.clear();
        }

        self.response.clear();
        self.response_headers.clear();
        self.response_cookies.clear();
        self.response_status = None;
        self.response_time = None;

        self.request_name = "Untitled-Request".to_string();

        self.show_response = false;

        self.unfocus();
        self.url_input.focus();
    }

    fn clear_request_data(&mut self) {
        self.url_input.clear();
        self.method_input.set_text("GET".to_string());

        for editor in &mut self.editors {
            editor.clear();
        }

        self.response.clear();
        self.response_headers.clear();
        self.response_cookies.clear();
        self.response_status = None;
        self.response_time = None;

        self.show_response = false;
    }

    pub fn unfocus(&mut self) {
        for editor in &mut self.editors {
            editor.unfocus();
        }
        self.response.unfocus();
        self.response_headers.unfocus();
        self.response_cookies.unfocus();
        self.url_input.unfocus();
        self.method_input.unfocus();
        self.collections.unfocus();
        self.curl_popup.unfocus();
        self.search_popup.unfocus();
        self.save_popup.unfocus();
    }

    pub async fn start_http_request(&mut self) -> color_eyre::Result<()> {
        self.is_loading = true;
        self.show_response = true;

        let url = self.url_input.get_text();
        let method = self.method_input.get_text();

        if url.is_empty() {
            self.response.set_text("Error: URL is required".to_string());
            self.response_status = Some("Error".to_string());
            self.response_time = None;
            self.is_loading = false;
            return Ok(());
        }

        let body = self.editors[2].get_text();
        let headers_text = self.editors[1].get_text();
        let sender = self.events.get_sender();

        tokio::spawn(async move {
            let start_time = Instant::now();
            let client = reqwest::Client::new();

            let request_builder = match method.to_uppercase().as_str() {
                "GET" => client.get(&url),
                "POST" => {
                    let mut builder = client.post(&url);
                    if !body.is_empty() {
                        builder = builder.body(body);
                    }
                    builder
                }
                "PUT" => {
                    let mut builder = client.put(&url);
                    if !body.is_empty() {
                        builder = builder.body(body);
                    }
                    builder
                }
                "DELETE" => client.delete(&url),
                "PATCH" => {
                    let mut builder = client.patch(&url);
                    if !body.is_empty() {
                        builder = builder.body(body);
                    }
                    builder
                }
                _ => client.get(&url),
            };

            let mut final_builder = request_builder;
            if !headers_text.is_empty() {
                for line in headers_text.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        final_builder = final_builder.header(key.trim(), value.trim());
                    }
                }
            }

            let response_data = match final_builder.send().await {
                Ok(response) => {
                    let elapsed = start_time.elapsed();
                    let status = response.status();
                    let status_str = Some(format!("{}", status.as_u16()));

                    let mut headers_text = String::new();
                    for (name, value) in response.headers() {
                        headers_text.push_str(&format!(
                            "{}: {}\n",
                            name,
                            value.to_str().unwrap_or("")
                        ));
                    }

                    let body = match response.text().await {
                        Ok(body) => {
                            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body)
                            {
                                serde_json::to_string_pretty(&json_value).unwrap_or(body)
                            } else {
                                body
                            }
                        }
                        Err(e) => format!("Error reading response body: {}", e),
                    };

                    HttpResponseData {
                        status: status_str,
                        response_time: Some(elapsed.as_millis()),
                        body,
                        headers: headers_text,
                    }
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    HttpResponseData {
                        status: Some("Error".to_string()),
                        response_time: Some(elapsed.as_millis()),
                        body: format!("Request failed: {}", e),
                        headers: String::new(),
                    }
                }
            };

            let _ = sender.send(crate::event::Event::App(AppEvent::HttpResponse(
                response_data,
            )));
        });

        Ok(())
    }

    pub fn handle_http_response(&mut self, response_data: HttpResponseData) {
        self.is_loading = false;
        self.response_status = response_data.status;
        self.response_time = response_data.response_time;
        self.response.set_text(response_data.body);
        self.response_headers.set_text(response_data.headers);

        self.unfocus();
        self.current_response_focus = 0;
        self.response.focus();
    }

    pub async fn save_renamed_request(&mut self) -> color_eyre::Result<()> {
        if self.rename_input.trim().is_empty() {
            self.is_renaming = false;
            return Ok(());
        }

        let old_name = &self.request_name;
        let new_name = self.rename_input.trim();

        if old_name != "Untitled-Request" && old_name != new_name {
            let old_path = format!("{}.curl", old_name);
            let new_path = format!("{}.curl", new_name);

            if std::path::Path::new(&old_path).exists() {
                if let Err(e) = std::fs::rename(&old_path, &new_path) {
                    self.save_error = Some(format!("Failed to rename file: {}", e));
                    self.is_renaming = false;
                    return Ok(());
                }
            }
        }

        self.request_name = new_name.to_string();
        self.is_renaming = false;
        self.rename_input.clear();
        self.save_error = None;

        self.collections.refresh();

        Ok(())
    }

    pub fn load_request_from_file(
        &mut self,
        file_path: &std::path::Path,
    ) -> color_eyre::Result<()> {
        if let Some(file_name) = file_path.file_stem() {
            if let Some(name_str) = file_name.to_str() {
                self.request_name = name_str.to_string();
            }
        }

        let content = std::fs::read_to_string(file_path)?;

        self.parse_curl_command(&content);

        Ok(())
    }

    pub fn parse_curl_command(&mut self, curl_command: &str) {
        match self.parse_curl_to_request(curl_command) {
            Ok(request) => {
                self.clear_request_data();

                self.url_input.set_text(request.url);

                self.method_input.set_text(request.method);

                let headers: Vec<String> = request
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                if !headers.is_empty() {
                    self.editors[1].set_text(headers.join("\n"));
                }

                if !request.body.is_empty() {
                    self.editors[2].set_text(request.body);
                }

                self.unfocus();
                self.url_input.focus();
            }
            Err(e) => {
                self.save_error = Some(format!("Failed to parse curl command: {}", e));
            }
        }
    }

    fn parse_curl_to_request(&self, curl_command: &str) -> Result<ParsedRequest, String> {
        let tokens = self.tokenize_curl_command(curl_command);
        self.parse_curl_tokens(tokens)
    }

    fn tokenize_curl_command(&self, cmd: &str) -> Vec<String> {
        let mut result = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;
        let mut chars = cmd.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                _ if escaped => {
                    result.push(ch);
                    escaped = false;
                }
                '\\' => {
                    if in_single_quote || in_double_quote {
                        result.push(ch);
                    } else {
                        if let Some(&'\n') = chars.peek() {
                            chars.next();
                            result.push(' ');
                        } else {
                            escaped = true;
                        }
                    }
                }
                '\'' if !in_double_quote => {
                    result.push(ch);
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    result.push(ch);
                    in_double_quote = !in_double_quote;
                }
                '\n' | '\r' => {
                    if in_single_quote || in_double_quote {
                        result.push(ch);
                    } else {
                        result.push(' ');
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        let cmd = if !result.trim_start().starts_with("curl ") {
            format!("curl {}", result.trim())
        } else {
            result
        };

        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        for ch in cmd.chars() {
            match ch {
                _ if escaped => {
                    current_token.push(ch);
                    escaped = false;
                }
                '\\' => {
                    current_token.push(ch);
                    escaped = true;
                }
                '\'' if !in_double_quote => {
                    current_token.push(ch);
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    current_token.push(ch);
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' if !in_single_quote && !in_double_quote => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    fn parse_curl_tokens(&self, args: Vec<String>) -> Result<ParsedRequest, String> {
        if args.is_empty() || args[0] != "curl" {
            return Err("Not a curl command".to_string());
        }

        let mut request = ParsedRequest {
            method: "GET".to_string(),
            url: String::new(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: String::new(),
            content_type: String::new(),
        };

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];

            if !arg.starts_with('-') && (arg.starts_with("http") || arg.contains("http")) {
                let url = self.remove_quotes(arg);
                request.url = url.clone();

                if let Some(question_mark) = url.find('?') {
                    let (base_url, query_string) = url.split_at(question_mark);
                    request.url = base_url.to_string();

                    let query_string = &query_string[1..];
                    for pair in query_string.split('&') {
                        if let Some(eq_pos) = pair.find('=') {
                            let (key, value) = pair.split_at(eq_pos);
                            let value = &value[1..];
                            request
                                .query_params
                                .insert(self.url_decode(key), self.url_decode(value));
                        }
                    }
                }
                i += 1;
                continue;
            }

            if (arg == "-X" || arg == "--request") && i + 1 < args.len() {
                request.method = args[i + 1].clone();
                i += 2;
                continue;
            }

            if (arg == "-H" || arg == "--header") && i + 1 < args.len() {
                let header_line = self.remove_quotes(&args[i + 1]);
                if let Some(colon_pos) = header_line.find(':') {
                    let (key, value) = header_line.split_at(colon_pos);
                    let value = &value[1..];
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();

                    if key.to_lowercase() == "content-type" {
                        request.content_type = value.clone();
                    }

                    request.headers.insert(key, value);
                }
                i += 2;
                continue;
            }

            if (arg == "-d" || arg == "--data" || arg == "--data-raw") && i + 1 < args.len() {
                request.body = self.remove_quotes(&args[i + 1]);

                if request.method == "GET" {
                    request.method = "POST".to_string();
                }

                if request.content_type.is_empty() {
                    request.content_type = "application/x-www-form-urlencoded".to_string();
                    request.headers.insert(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    );
                }

                i += 2;
                continue;
            }

            if arg == "--json" && i + 1 < args.len() {
                request.body = self.remove_quotes(&args[i + 1]);
                request.content_type = "application/json".to_string();
                request
                    .headers
                    .insert("Content-Type".to_string(), "application/json".to_string());

                if request.method == "GET" {
                    request.method = "POST".to_string();
                }

                i += 2;
                continue;
            }

            if (arg == "-F" || arg == "--form") && i + 1 < args.len() {
                if request.content_type.is_empty() {
                    request.content_type = "multipart/form-data".to_string();
                    request.headers.insert(
                        "Content-Type".to_string(),
                        "multipart/form-data".to_string(),
                    );
                }

                if request.method == "GET" {
                    request.method = "POST".to_string();
                }

                i += 2;
                continue;
            }

            i += 1;
        }

        Ok(request)
    }

    fn remove_quotes(&self, s: &str) -> String {
        if s.len() < 2 {
            return s.to_string();
        }

        let chars: Vec<char> = s.chars().collect();
        if (chars[0] == '"' && chars[chars.len() - 1] == '"')
            || (chars[0] == '\'' && chars[chars.len() - 1] == '\'')
        {
            let mut result = s[1..s.len() - 1].to_string();

            result = result.replace(r#"\""#, r#"""#);
            result = result.replace(r"\'", "'");
            result = result.replace(r"\\", r"\");

            return result;
        }

        s.to_string()
    }

    fn url_decode(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                        result.push(byte as char);
                    } else {
                        result.push(ch);
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub async fn save_request_to_file(
        &mut self,
        file_path: &std::path::Path,
    ) -> color_eyre::Result<()> {
        let url = self.url_input.get_text();
        let method = self.method_input.get_text();
        let headers = self.editors[1].get_text();
        let body = self.editors[2].get_text();

        let mut curl_command = format!("curl -X {} '{}'", method, url);

        if !headers.is_empty() {
            for line in headers.lines() {
                if !line.trim().is_empty() {
                    curl_command.push_str(&format!(" \\\n  -H '{}'", line.trim()));
                }
            }
        }

        if !body.is_empty() {
            curl_command.push_str(&format!(" \\\n  -d '{}'", body.replace("'", "'\\''")));
        }

        std::fs::write(file_path, curl_command)?;

        Ok(())
    }
}

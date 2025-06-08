use core::fmt;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

use crate::themes::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Operator(char),
}

impl Mode {
    // pub fn block<'a>(&self) -> Block<'a> {
    //     // let help = match self {
    //     //     Self::Normal => "type q to quit, type i to enter insert mode",
    //     //     Self::Insert => "type Esc to back to normal mode",
    //     //     Self::Visual => "type y to yank, type d to delete, type Esc to back to normal mode",
    //     //     Self::Operator(_) => "move cursor to apply operator",
    //     // };
    //     // let title = format!("{} MODE", self);
    //
    //     let block = Block::bordered()
    //         .style(Style::default().fg(Color::Rgb(93, 93, 93)))
    //         .border_type(BorderType::Rounded);
    //
    //     return block;
    // }

    // pub fn cursor_style(&self) -> Style {
    //     let color = match self {
    //         Self::Normal => Color::Reset,
    //         Self::Insert => Color::LightBlue,
    //         Self::Visual => Color::LightYellow,
    //         Self::Operator(_) => Color::LightGreen,
    //     };
    //     Style::default().fg(color).add_modifier(Modifier::REVERSED)
    // }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Visual => write!(f, "VISUAL"),
            Self::Operator(c) => write!(f, "OPERATOR({})", c),
        }
    }
}

// How the Vim emulation state transitions
pub enum Transition {
    Nop,
    Mode(Mode),
    Pending(Input),
    Quit,
}

// State of Vim emulation
#[derive(Debug, Clone)]
pub struct Vim {
    pub mode: Mode,
    pending: Input, // Pending input to handle a sequence with two keys like gg
    block: Block<'static>,
    pub textarea: TextArea<'static>,
    focused: bool,
    theme: &'static Theme,
}

impl Vim {
    pub fn new(mode: Mode, theme: &'static Theme) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default().fg(theme.foreground));
        textarea.set_style(Style::default().fg(theme.foreground));
        textarea.set_cursor_style(Style::default().bg(theme.cursor));
        textarea.set_selection_style(Style::default().bg(theme.selection));
        Self {
            mode,
            pending: Input::default(),
            block: Block::default(),
            textarea: textarea,
            focused: false,
            theme,
        }
    }

    pub fn with_pending(self, pending: Input) -> Self {
        Self {
            mode: self.mode,
            pending,
            block: self.block,
            textarea: self.textarea,
            focused: self.focused,
            theme: self.theme,
        }
    }

    // pub fn handle_key(&mut self, key: KeyEvent) -> bool {}

    pub fn transition(&mut self, input: Input) -> Transition {
        if input.key == Key::Null {
            return Transition::Nop;
        }

        match self.mode {
            Mode::Normal | Mode::Visual | Mode::Operator(_) => {
                match input {
                    Input {
                        key: Key::Char('h'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::Back),
                    Input {
                        key: Key::Char('j'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::Down),
                    Input {
                        key: Key::Char('k'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::Up),
                    Input {
                        key: Key::Char('l'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::Forward),
                    Input {
                        key: Key::Char('w'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::WordForward),
                    Input {
                        key: Key::Char('e'),
                        ctrl: false,
                        ..
                    } => {
                        self.textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(self.mode, Mode::Operator(_)) {
                            self.textarea.move_cursor(CursorMove::Forward); // Include the text under the cursor
                        }
                    }
                    Input {
                        key: Key::Char('b'),
                        ctrl: false,
                        ..
                    } => self.textarea.move_cursor(CursorMove::WordBack),
                    Input {
                        key: Key::Char('^'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::Head),
                    Input {
                        key: Key::Char('$'),
                        ..
                    } => self.textarea.move_cursor(CursorMove::End),
                    Input {
                        key: Key::Char('D'),
                        ..
                    } => {
                        self.textarea.delete_line_by_end();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('C'),
                        ..
                    } => {
                        self.textarea.delete_line_by_end();
                        self.textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('p'),
                        ..
                    } => {
                        self.textarea.paste();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    } => {
                        self.textarea.undo();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('r'),
                        ctrl: true,
                        ..
                    } => {
                        self.textarea.redo();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('x'),
                        ..
                    } => {
                        self.textarea.delete_next_char();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('i'),
                        ..
                    } => {
                        self.textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('a'),
                        ..
                    } => {
                        self.textarea.cancel_selection();
                        self.textarea.move_cursor(CursorMove::Forward);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('A'),
                        ..
                    } => {
                        self.textarea.cancel_selection();
                        self.textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('o'),
                        ..
                    } => {
                        self.textarea.move_cursor(CursorMove::End);
                        self.textarea.insert_newline();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('O'),
                        ..
                    } => {
                        self.textarea.move_cursor(CursorMove::Head);
                        self.textarea.insert_newline();
                        self.textarea.move_cursor(CursorMove::Up);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('I'),
                        ..
                    } => {
                        self.textarea.cancel_selection();
                        self.textarea.move_cursor(CursorMove::Head);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('q'),
                        ..
                    } => return Transition::Quit,
                    Input {
                        key: Key::Char('e'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll((1, 0)),
                    Input {
                        key: Key::Char('y'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll((-1, 0)),
                    Input {
                        key: Key::Char('d'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll(Scrolling::HalfPageDown),
                    Input {
                        key: Key::Char('u'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll(Scrolling::HalfPageUp),
                    Input {
                        key: Key::Char('f'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll(Scrolling::PageDown),
                    Input {
                        key: Key::Char('b'),
                        ctrl: true,
                        ..
                    } => self.textarea.scroll(Scrolling::PageUp),
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        self.textarea.start_selection();
                        return Transition::Mode(Mode::Visual);
                    }
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        self.textarea.move_cursor(CursorMove::Head);
                        self.textarea.start_selection();
                        self.textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Visual);
                    }
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        self.textarea.cancel_selection();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    } if matches!(
                        self.pending,
                        Input {
                            key: Key::Char('g'),
                            ctrl: false,
                            ..
                        }
                    ) =>
                    {
                        self.textarea.move_cursor(CursorMove::Top)
                    }
                    Input {
                        key: Key::Char('G'),
                        ctrl: false,
                        ..
                    } => self.textarea.move_cursor(CursorMove::Bottom),
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Operator(c) => {
                        // Handle yy, dd, cc. (This is not strictly the same behavior as Vim)
                        self.textarea.move_cursor(CursorMove::Head);
                        self.textarea.start_selection();
                        let cursor = self.textarea.cursor();
                        self.textarea.move_cursor(CursorMove::Down);
                        if cursor == self.textarea.cursor() {
                            self.textarea.move_cursor(CursorMove::End); // At the last line, move to end of the line instead
                        }
                    }
                    Input {
                        key: Key::Char(op @ ('y' | 'd' | 'c')),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        self.textarea.start_selection();
                        return Transition::Mode(Mode::Operator(op));
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        self.textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        self.textarea.copy();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        self.textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        self.textarea.cut();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('c'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        self.textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        self.textarea.cut();
                        return Transition::Mode(Mode::Insert);
                    }
                    input => return Transition::Pending(input),
                }

                // Handle the pending operator
                match self.mode {
                    Mode::Operator('y') => {
                        self.textarea.copy();
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('d') => {
                        self.textarea.cut();
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('c') => {
                        self.textarea.cut();
                        Transition::Mode(Mode::Insert)
                    }
                    _ => Transition::Nop,
                }
            }
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => Transition::Mode(Mode::Normal),
                input => {
                    self.textarea.input(input); // Use default key mappings in insert mode
                    Transition::Mode(Mode::Insert)
                }
            },
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.textarea.render(area, buf);
    }

    pub fn focus(&mut self) {
        self.textarea
            .set_cursor_style(Style::default().fg(Color::Black).bg(self.theme.cursor));
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
}

// impl RequestInput {
//     pub fn new() -> Self {
//         let mut textarea = TextArea::default();
//         // textarea.set_style(Style::default().bg(Color::Rgb(14, 18, 40)));
//         textarea.set_block(Mode::Normal.block());
//         textarea.set_cursor_style(Mode::Normal.cursor_style());
//         let mut vim = Vim::new(Mode::Normal);
//         textarea.set_cursor_line_style(Style::default().fg(Color::White));
//         textarea.set_cursor_style(Style::default().fg(Color::White));
//         let block = Block::bordered()
//             .style(Style::default().fg(Color::Rgb(93, 93, 93)))
//             .border_type(BorderType::Rounded);
//         textarea.set_block(block);
//         Self {
//             textarea,
//             focused: false,
//         }
//     }
//
//
//     pub fn get_input(self) -> String {
//         self.textarea.lines()[0].clone()
//     }
//
//     pub fn render(&self, area: Rect, buf: &mut Buffer) {
//         self.textarea.render(area, buf);
//     }
//
//     pub fn focus(&mut self) {
//         self.textarea
//             .set_cursor_style(Style::default().bg(Color::Gray));
//         self.textarea.move_cursor(CursorMove::End);
//         self.focused = true;
//     }
//
//     pub fn unfocus(&mut self) {
//         self.textarea
//             .set_cursor_style(Style::default().fg(Color::White));
//         self.focused = false;
//     }
//
//     pub fn is_focused(&self) -> bool {
//         self.focused
//     }

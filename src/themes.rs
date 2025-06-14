use ratatui::style::Color;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub accent: Color,
    pub cursor: Color,
    pub selection: Color,
    pub list_background: Color,
    pub method_cursor: Color,
    pub tree_lines: Color,
}

pub const ARCHER: Theme = Theme {
    background: Color::Rgb(33, 33, 33),
    foreground: Color::White,
    border: Color::Rgb(120, 120, 120),
    accent: Color::Rgb(250, 178, 131),
    cursor: Color::Rgb(250, 178, 131),
    selection: Color::Rgb(69, 64, 61),
    list_background: Color::Rgb(38, 38, 38),
    method_cursor: Color::Rgb(250, 178, 255),
    tree_lines: Color::Rgb(74, 68, 64),
};

pub const GRUVBOX: Theme = Theme {
    background: Color::Rgb(40, 40, 40),       // gruvbox dark bg0
    foreground: Color::Rgb(235, 219, 178),    // gruvbox light fg
    border: Color::Rgb(146, 131, 116),        // gruvbox gray
    accent: Color::Rgb(254, 128, 25),         // gruvbox orange
    cursor: Color::Rgb(254, 128, 25),         // gruvbox orange
    selection: Color::Rgb(80, 73, 69),        // gruvbox dark bg2
    list_background: Color::Rgb(60, 56, 54),  // gruvbox dark bg1
    method_cursor: Color::Rgb(211, 134, 155), // gruvbox purple
    tree_lines: Color::Rgb(168, 153, 132),    // gruvbox light gray
};

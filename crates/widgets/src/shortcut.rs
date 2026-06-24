use std::ops::Range;

use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    style::{Color, Style},
};

pub struct Shortcut<'a> {
    name: &'a str,
    key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }
}

type Name = Range<usize>;
type Key = Range<usize>;

pub struct Shortcuts {
    formatter: utils::Formatter,
    shortcuts: Vec<(Name, Key)>,
    name_color: Color,
    key_color: Color,
    alignment: HorizontalAlignment,
}

impl Shortcuts {
    pub const fn new() -> Self {
        Self {
            formatter: utils::Formatter::new(),
            shortcuts: Vec::new(),
            name_color: Color::Reset,
            key_color: Color::Indexed(240),
            alignment: HorizontalAlignment::Center,
        }
    }

    pub const fn with_alignment(mut self, align: HorizontalAlignment) -> Self {
        self.set_alignment(align);
        self
    }

    pub const fn with_colors(mut self, name: Color, key: Color) -> Self {
        self.set_colors(name, key);
        self
    }

    pub const fn set_alignment(&mut self, align: HorizontalAlignment) -> &mut Self {
        self.alignment = align;
        self
    }

    pub const fn set_colors(&mut self, name: Color, key: Color) -> &mut Self {
        self.name_color = name;
        self.key_color = key;
        self
    }

    pub fn push(&mut self, shortcut: Shortcut<'_>) {
        let name = self.formatter.push_str(shortcut.name);
        let key = self.formatter.push_str(shortcut.key);
        self.shortcuts.push((name, key));
    }

    pub fn push_iter<'a>(&mut self, name: impl IntoIterator<Item = &'a str>, key: &str) {
        let name = self.formatter.extend(name);
        let key = self.formatter.push_str(key);
        self.shortcuts.push((name, key));
    }

    pub fn extend<'a>(&mut self, shortcuts: impl IntoIterator<Item = Shortcut<'a>>) {
        self.shortcuts.extend(shortcuts.into_iter().map(|s| {
            let name = self.formatter.push_str(s.name);
            let key = self.formatter.push_str(s.key);
            (name, key)
        }));
    }

    pub fn clear(&mut self) {
        self.formatter.clear();
        self.shortcuts.clear();
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let alignment = match self.alignment {
            HorizontalAlignment::Left => None,
            HorizontalAlignment::Center => Some(crate::Alignment::CenterHorizontal),
            HorizontalAlignment::Right => Some(crate::Alignment::Right),
        };
        let len = self.shortcuts.len();
        crate::print_texts_with_styles(
            area,
            buf,
            self.shortcuts
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, (name, key))| {
                    let name = self.formatter.slice(name);
                    let name_style = Style::new().fg(self.name_color);
                    let key = self.formatter.slice(key);
                    let key_style = Style::new().fg(self.key_color);
                    let is_last = i + 1 == len;
                    let gap = if is_last { "" } else { " " };
                    (
                        (key, key_style),
                        (" ", Style::new()),
                        (name, name_style),
                        (gap, Style::new()),
                    )
                })
                .flat_map(|(a, b, c, d)| [a, b, c, d]),
            None,
            alignment,
        );
    }
}

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
};

pub const STYLE_NONE: Style = Style::new();
pub const STYLE_BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
pub const STYLE_ITALIC: Style = Style::new().add_modifier(Modifier::ITALIC);

pub struct Shortcut<'a> {
    name: &'a str,
    key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }

    pub fn as_spans(self, accent_color: Color) -> [Span<'a>; 5] {
        [
            Span::raw(" "),
            Span::styled(self.key, accent_color),
            Span::raw(" "),
            Span::raw(self.name),
            Span::raw(" "),
        ]
    }
}

pub struct Shortcuts<'a>(Vec<Shortcut<'a>>);

impl<'a> Shortcuts<'a> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }
}

impl<'a> std::ops::Deref for Shortcuts<'a> {
    type Target = Vec<Shortcut<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for Shortcuts<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn _layout_center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    _layout_center_vertical(layout_center_horizontal(area, horizontal), vertical)
}

pub fn layout_center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn _layout_center_vertical(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

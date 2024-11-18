use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
};

pub const STYLE_NONE: Style = Style::new();
pub const STYLE_BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
pub const STYLE_ITALIC: Style = Style::new().add_modifier(Modifier::ITALIC);
pub const STYLE_LABEL: Style = Style::new().fg(Color::Gray);
pub const STYLE_CURSOR: Style = Style::new().bg(Color::Blue);
pub const STYLE_SELECTED: Style = Style::new().bg(Color::Blue);

pub const SHORTCUT_QUIT: Shortcut = Shortcut::new("Quit", "Esc");
pub const SHORTCUT_NEXT: Shortcut = Shortcut::new("Next", "Tab");
pub const SHORTCUT_PREV: Shortcut = Shortcut::new("Prev", "⇧Tab");
pub const SHORTCUT_SHOW: Shortcut = Shortcut::new("Show", "Space");
pub const SHORTCUT_YES: Shortcut = Shortcut::new("Yes", "y");
pub const SHORTCUT_NO: Shortcut = Shortcut::new("No", "n");
pub const SHORTCUT_EDIT: Shortcut = Shortcut::new("Edit", "e");
pub const SHORTCUT_DELETE: Shortcut = Shortcut::new("Delete", "Del");
pub const SHORTCUT_SAVE: Shortcut = Shortcut::new("Save", "^s");
pub const _SHORTCUT_CANCEL: Shortcut = Shortcut::new("Cancel", "^c");
pub const SHORTCUT_PREVIEW: Shortcut = Shortcut::new("Preview", "^p");
pub const _SHORTCUT_SCROLL: Shortcut = Shortcut::new("Scroll", "⇅");
pub const SHORTCUT_SKIP: Shortcut = Shortcut::new("Skip", "➝");
pub const SHORTCUT_BROWSE: Shortcut = Shortcut::new("Browse", "⇄");
pub const SHORTCUT_SORT: Shortcut = Shortcut::new("Sort", "s");

pub const MARGIN_CONTENT: Margin = Margin::new(2, 2);

pub struct Shortcut<'a> {
    pub name: &'a str,
    pub key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }

    pub fn as_spans(self) -> [Span<'a>; 5] {
        [
            Span::raw(" "),
            Span::styled(self.key, STYLE_LABEL),
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

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
};

pub const STYLE_LABEL: Style = Style::new().fg(Color::Gray);

pub const SHORTCUT_QUIT: Shortcut = Shortcut::new("Quit", "Esc");
pub const SHORTCUT_MENU: Shortcut = Shortcut::new("Menu", "Tab");
pub const SHORTCUT_EDIT: Shortcut = Shortcut::new("Edit", "e");
pub const SHORTCUT_DELETE: Shortcut = Shortcut::new("Delete", "Del");
pub const SHORTCUT_SAVE: Shortcut = Shortcut::new("Save", "^s");
pub const SHORTCUT_CANCEL: Shortcut = Shortcut::new("Cancel", "^c");
pub const SHORTCUT_PREVIEW: Shortcut = Shortcut::new("Preview", "^p");

pub struct Shortcut<'a> {
    pub name: &'a str,
    pub key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }
}

pub fn layout_center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    layout_center_vertical(layout_center_horizontal(area, horizontal), vertical)
}

pub fn layout_center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn layout_center_vertical(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

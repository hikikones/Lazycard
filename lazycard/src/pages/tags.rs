use database::TagId;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};
use widgets::{ListItem, Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput},
    settings::Colors,
    symbols,
};

pub struct TagsPage {
    tags: Vec<TagId>,
}

impl TagsPage {
    pub const fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn on_enter(&mut self) {}

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut TextSegment,
        shortcuts: &mut Shortcuts,
    ) {
        if self.tags.is_empty() {
            widgets::print_ascii(
                area,
                buf,
                "No tags",
                colors.neutral,
                Some(widgets::Alignment::Center),
            );
            return;
        }

        menu.push_str("Tags", colors.neutral);

        // Render

        // Shortcuts
        shortcuts.extend([
            Shortcut::new("New", "n"),
            Shortcut::new("Edit", "e"),
            Shortcut::new("Delete", symbols::DELETE),
        ]);
    }

    pub fn on_input(&mut self, input: AppInput) -> Action {
        let key = input.key_pressed();
        match key {
            KeyCode::Right => {
                //todo
            }
            KeyCode::Left => {
                //todo
            }
            KeyCode::Delete => {
                //todo
            }
            KeyCode::Char('n') => {
                //todo
            }
            KeyCode::Char('e') => {
                //todo
            }
            _ => {
                //todo?
            }
        }

        Action::None
    }

    pub fn on_exit(&self) {}
}

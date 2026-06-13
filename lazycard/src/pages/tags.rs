use std::ops::Range;

use database::{Database, TagId};
use ratatui::{
    buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::Style, text::Span,
    widgets::Widget,
};
use utils::Formatter;
use widgets::{Shortcut, Shortcuts, TextSegment, TokenItem, TokenList};

use crate::{
    app::{Action, AppInput},
    settings::Colors,
    symbols,
};

// TODO: Scrollbar.

pub struct TagsPage {
    tags: Vec<TagItem>,
    names: Formatter,
    list: TokenList,
}

impl TagsPage {
    pub const fn new() -> Self {
        Self {
            tags: Vec::new(),
            names: Formatter::new(),
            list: TokenList::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        db.get_tags_and_name(|id, name| {
            self.tags.push(TagItem {
                id,
                name: self.names.push_str(name),
                width: unicode_width::UnicodeWidthStr::width(name) as u16,
            });
        })
        .unwrap();
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut TextSegment,
        shortcuts: &mut Shortcuts,
    ) {
        menu.push_str("Tags", colors.neutral);

        if self.tags.is_empty() {
            widgets::print_ascii(
                area,
                buf,
                "No tags",
                colors.neutral,
                Some(widgets::Alignment::Center),
            );
            shortcuts.push(Shortcut::new("New", "n"));
            return;
        }

        self.list.render(
            area,
            buf,
            self.tags.iter(),
            |area, buf, tag, is_selected| {
                let style = if is_selected {
                    Style::new().fg(colors.primary)
                } else {
                    Style::new()
                };
                let name = self.names.slice(tag.name.clone());
                Span::styled(name, style).render(area, buf);
            },
        );

        shortcuts.extend([
            Shortcut::new("New", "n"),
            Shortcut::new("Edit", "e"),
            Shortcut::new("Delete", symbols::DELETE),
        ]);
    }

    pub fn on_input(&mut self, input: AppInput) -> Action {
        let key = input.key_pressed();

        match key {
            KeyCode::Delete => {
                //todo: delete
            }
            KeyCode::Char('n') => {
                //todo: new
            }
            KeyCode::Char('e') => {
                //todo: edit
            }
            _ => {
                if self.list.input(key, self.tags.iter()) {
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.tags.clear();
        self.names.clear();
    }
}

struct TagItem {
    id: TagId,
    name: Range<usize>,
    width: u16,
}

impl TokenItem for &TagItem {
    fn width(&self) -> u16 {
        self.width
    }
}

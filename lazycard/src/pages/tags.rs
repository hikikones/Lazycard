use std::ops::Range;

use database::{Database, TagId};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::Style};
use utils::Formatter;
use widgets::{Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput},
    settings::Colors,
    symbols,
};

pub struct TagsPage {
    tags: Vec<Badge>,
    index: usize,
    names: Formatter,
}

impl TagsPage {
    pub const fn new() -> Self {
        Self {
            tags: Vec::new(),
            index: 0,
            names: Formatter::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo: test remove
        for i in 0..150 {
            db.add_tag(&format!("tag_{i}")).unwrap();
        }

        db.get_tags_and_name(|id, name| {
            self.tags.push(Badge {
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

        // Render tags as a "chip/tag list"
        let (mut x, mut y) = (0, 0);

        for (i, tag) in self.tags.iter().enumerate() {
            if tag.width + x >= area.width {
                x = 0;
                if i > 0 {
                    y += 1;
                }
            }

            if y >= area.height {
                break;
            }

            let style = if self.index == i {
                Style::new().fg(colors.primary)
            } else {
                Style::new()
            };

            buf.set_stringn(
                area.x + x,
                area.y + y,
                self.names.slice(tag.name.clone()),
                tag.width as usize,
                style,
            );

            const GAP: u16 = 2;

            x += tag.width + GAP;
        }

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
                if self.index < self.tags.len().saturating_sub(1) {
                    self.index += 1;
                    return Action::Render;
                }
            }
            KeyCode::Left => {
                if self.index > 0 {
                    self.index -= 1;
                    return Action::Render;
                }
            }
            KeyCode::Down => {
                //todo
            }
            KeyCode::Up => {
                //todo
            }
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
                //todo?
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.tags.clear();
        self.names.clear();
    }
}

struct Badge {
    id: TagId,
    name: Range<usize>,
    width: u16,
}

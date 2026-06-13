use std::ops::Range;

use database::{Database, TagId};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Rect, Size},
    style::Style,
    text::Span,
    widgets::Widget,
};
use utils::Formatter;
use widgets::{Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput},
    settings::Colors,
    symbols,
};

pub struct TagsPage {
    tags: Vec<Badge>,
    names: Formatter,
    index: usize,
    col: u16,
    row: u16,
    scroll: u16,
    list: TagsList,
}

impl TagsPage {
    pub const fn new() -> Self {
        Self {
            tags: Vec::new(),
            names: Formatter::new(),
            index: 0,
            col: 0,
            row: 0,
            scroll: 0,
            list: TagsList::new(),
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

        self.list.render(
            area,
            buf,
            self.tags.iter(),
            |area, buf, badge, is_selected| {
                //todo
                let style = if is_selected {
                    Style::new().fg(colors.primary)
                } else {
                    Style::new()
                };
                let name = self.names.slice(badge.name.clone());
                Span::styled(name, style).render(area, buf);
            },
        );

        // // Render tags as a "chip/tag list"
        // let (mut col, mut row) = (0, 0);

        // for (i, tag) in self.tags.iter().enumerate() {
        //     let style = if self.index == i {
        //         self.col = col;
        //         self.row = row;
        //         Style::new().fg(colors.primary)
        //     } else {
        //         Style::new()
        //     };

        //     if row >= self.scroll && row < area.height + self.scroll {
        //         buf.set_stringn(
        //             area.x + col,
        //             area.y + row,
        //             self.names.slice(tag.name.clone()),
        //             tag.width as usize,
        //             style,
        //         );
        //     }

        //     const GAP: u16 = 2;
        //     col += tag.width + GAP;

        //     if col + tag.width >= area.width {
        //         col = 0;
        //         // if i > 0 {
        //         // }
        //         row += 1;
        //     }

        //     if row >= area.height + self.scroll {
        //         break;
        //     }
        // }

        // Shortcuts
        shortcuts.extend([
            Shortcut::new("New", "n"),
            Shortcut::new("Edit", "e"),
            Shortcut::new("Delete", symbols::DELETE),
        ]);
    }

    pub fn on_input(&mut self, input: AppInput) -> Action {
        let key = input.key_pressed();

        if self.list.input(key) {
            return Action::Render;
        }

        return Action::None;

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

impl TagItem for &Badge {
    fn width(&self) -> u16 {
        self.width
    }
}

pub struct TagsList {
    index: usize,
    index_col: u16,
    index_row: u16,
    scroll: u16,
    size: Size,
    total_lines: u16,
    total_items: usize,
}

pub trait TagItem {
    fn width(&self) -> u16;
}

impl TagsList {
    pub const fn new() -> Self {
        Self {
            index: 0,
            index_col: 0,
            index_row: 0,
            scroll: 0,
            size: Size::ZERO,
            total_lines: 0,
            total_items: 0,
        }
    }

    pub fn input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Right => {
                if self.index < self.total_items.saturating_sub(1) {
                    self.index += 1;
                    return true;
                }
            }
            KeyCode::Left => {
                if self.index > 0 {
                    self.index -= 1;
                    return true;
                }
            }
            KeyCode::Down => {
                //todo
            }
            KeyCode::Up => {
                //todo
            }

            _ => {}
        }

        false
    }

    pub fn render<T: TagItem>(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        items: impl IntoIterator<Item = T, IntoIter: Clone>,
        mut render_item: impl FnMut(Rect, &mut Buffer, T, bool),
    ) {
        let items = items.into_iter();

        // Process col, row and totals
        const GAP: u16 = 2;
        // if self.size.width != area.width {
        self.total_items = 0;

        let (w, mut x, mut y) = (area.width, 0, 0);
        for (i, item) in items.clone().enumerate() {
            if self.index == i {
                self.index_col = x;
                self.index_row = y;
            }

            x += item.width() + GAP;

            if x >= w {
                x = 0;
                y += 1;
            }

            self.total_items += 1;
        }

        self.total_lines = y + 1;
        // }

        // if self.index == 14 {
        //     panic!("INDEX_ROW: {}", self.index_row);
        // }

        self.size = Size::from(area);

        // Determine scroll
        let scroll = if self.size.height != area.height {
            // Refresh scroll on window resize
            0
        } else {
            self.scroll
        };
        self.scroll = widgets::calculate_scroll(
            self.total_lines as usize,
            area.height,
            self.index_row as usize,
            // self.index, // TODO: should be ROW
            scroll as usize,
            0,
            0,
            0,
        ) as u16;

        // Render
        let (mut col, mut row) = (0, 0);
        for (i, item) in items.enumerate() {
            let item_width = item.width();

            if row >= self.scroll && row < area.height + self.scroll {
                let area = Rect {
                    x: area.x + col,
                    y: area.y + row.saturating_sub(self.scroll),
                    width: item_width.min(area.width.saturating_sub(col + 1)),
                    height: 1,
                };
                render_item(area, buf, item, self.index == i);
            }

            col += item_width + GAP;
            if col >= area.width {
                col = 0;
                row += 1;
            }

            // TODO: break early?
        }

        // if row >= self.scroll && row < area.height + self.scroll {
        //     buf.set_stringn(
        //         area.x + col,
        //         area.y + row,
        //         self.names.slice(tag.name.clone()),
        //         tag.width as usize,
        //         style,
        //     );
        // }

        // const GAP: u16 = 2;
        // col += tag.width + GAP;

        // if col + tag.width >= area.width {
        //     col = 0;
        //     // if i > 0 {
        //     // }
        //     row += 1;
        // }

        // if row >= area.height + self.scroll {
        //     break;
        // }

        // // Make sure index and selector is not out of bounds
        // let max_idx = items.len().saturating_sub(1);
        // self.index = self.index.min(max_idx);
        // self.selector = self.selector.map(|selector| selector.min(max_idx));

        // // Determine scroll
        // let scroll = if self.height != area.height {
        //     // Refresh scroll on window resize
        //     0
        // } else {
        //     self.scroll
        // };
        // self.scroll = utils::calculate_scroll(
        //     items.len(),
        //     area.height,
        //     self.index,
        //     scroll,
        //     self.margin_top,
        //     self.margin_bottom,
        //     self.padding_bottom,
        // );

        // self.len = items.len();
        // self.height = area.height;

        // // Render
        // let height = area.height as usize;
        // let scrollable = items.len() > height;

        // if scrollable {
        //     let scrollbar = Rect {
        //         x: area.x + area.width.saturating_sub(1),
        //         width: 1,
        //         ..area
        //     };
        //     area.width = area.width.saturating_sub(3);
        //     utils::render_scrollbar(
        //         scrollbar,
        //         buf,
        //         items.len(),
        //         self.scroll,
        //         self.thumb_color,
        //         self.track_color,
        //     );
        // }

        // let selection = self.selection_inclusive();
        // let mut line = Rect { height: 1, ..area };

        // items
        //     .enumerate()
        //     .skip(self.scroll)
        //     .take(height)
        //     .for_each(|(i, item)| {
        //         let list_item = if i == self.index {
        //             ListItem::Selected
        //         } else if selection.contains(&i) {
        //             ListItem::Selection
        //         } else {
        //             ListItem::Normal
        //         };

        //         render_item(line, buf, item, list_item);

        //         line.y += 1;
        //     });
    }
}

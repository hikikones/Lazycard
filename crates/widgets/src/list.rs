use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};

use crate::{Scrollbar, ScrollbarColors};

// TODO: Add render_with_splits or with_splits for horizontal split of the area.

pub struct List {
    index: usize,
    selector: Option<usize>,
    scroll: usize,
    margin_top: usize,
    margin_bottom: usize,
    padding_bottom: usize,
    scrollbar: Option<ScrollbarColors>,
    len: usize,
    height: u16,
}

pub enum ListMove {
    Up(usize),
    Down(usize),
    PageUp,
    PageDown,
    Start,
    End,
    Custom(usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ListItem {
    Selected,
    Selection,
    Normal,
}

impl List {
    pub const fn new() -> Self {
        Self {
            index: 0,
            selector: None,
            scroll: 0,
            margin_top: 0,
            margin_bottom: 0,
            padding_bottom: 0,
            scrollbar: None,
            len: 0,
            height: 0,
        }
    }

    pub const fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub const fn with_margins(mut self, top: usize, bottom: usize) -> Self {
        self.set_margins(top, bottom);
        self
    }

    pub const fn with_padding(mut self, bottom: usize) -> Self {
        self.set_padding(bottom);
        self
    }

    pub const fn index(&self) -> usize {
        self.index
    }

    pub const fn selector(&self) -> Option<usize> {
        self.selector
    }

    pub const fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn selection(&self) -> Option<std::ops::Range<usize>> {
        self.selector
            .and_then(|selector| match self.index.cmp(&selector) {
                std::cmp::Ordering::Less => Some((self.index + 1)..(selector + 1)),
                std::cmp::Ordering::Greater => Some(selector..self.index),
                std::cmp::Ordering::Equal => None,
            })
    }

    pub fn selection_inclusive(&self) -> std::ops::RangeInclusive<usize> {
        self.selector
            .map(|selector| {
                if self.index < selector {
                    self.index..=selector
                } else {
                    selector..=self.index
                }
            })
            .unwrap_or(self.index..=self.index)
    }

    pub const fn set_margins(&mut self, top: usize, bottom: usize) -> &mut Self {
        self.margin_top = top;
        self.margin_bottom = bottom;
        self
    }

    pub const fn set_padding(&mut self, bottom: usize) -> &mut Self {
        self.padding_bottom = bottom;
        self
    }

    pub const fn set_scrollbar(&mut self, colors: ScrollbarColors) -> &mut Self {
        self.scrollbar = Some(colors);
        self
    }

    pub const fn set_index(&mut self, i: usize) -> &mut Self {
        self.index = i;
        self
    }

    pub const fn set_selector(&mut self, s: Option<usize>) -> &mut Self {
        self.selector = s;
        self
    }

    pub fn move_index(&mut self, lm: ListMove, shift: bool) -> bool {
        match lm {
            ListMove::Up(n) => self.set_index_and_selector(self.index.saturating_sub(n), shift),
            ListMove::Down(n) => self.set_index_and_selector(self.index + n, shift),
            ListMove::PageUp => {
                let n = self.height as usize;
                self.set_index_and_selector(self.index.saturating_sub(n), shift)
            }
            ListMove::PageDown => {
                let n = self.height as usize;
                self.set_index_and_selector(self.index + n, shift)
            }
            ListMove::Start => self.set_index_and_selector(0, shift),
            ListMove::End => self.set_index_and_selector(usize::MAX, shift),
            ListMove::Custom(i) => self.set_index_and_selector(i, shift),
        }
    }

    pub fn move_selection_up(&mut self) -> bool {
        let Some(selector) = self.selector else {
            return self.set_index_and_selector(self.index.saturating_sub(1), false);
        };

        let index = self.index;
        let i = index.saturating_sub(1);
        let s = selector.saturating_sub(1);

        if i == index || s == selector {
            return false;
        }

        self.index = i;
        self.selector = Some(s);
        true
    }

    pub fn move_selection_down(&mut self) -> bool {
        let Some(selector) = self.selector else {
            return self.set_index_and_selector(self.index + 1, false);
        };

        let index = self.index;
        let max_index = self.len.saturating_sub(1);
        let i = usize::min(index + 1, max_index);
        let s = usize::min(selector + 1, max_index);

        if i == index || s == selector {
            return false;
        }

        self.index = i;
        self.selector = Some(s);
        true
    }

    pub fn select_all(&mut self) -> bool {
        let old_index = self.index;
        let old_selector = self.selector;

        self.index = 0;
        self.selector = Some(self.len.saturating_sub(1));
        self.selector.take_if(|s| *s == self.index);

        old_index != self.index || old_selector != self.selector
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Down => self.move_index(ListMove::Down(1), shift),
            KeyCode::Up => self.move_index(ListMove::Up(1), shift),
            KeyCode::PageDown => self.move_index(ListMove::PageDown, shift),
            KeyCode::PageUp => self.move_index(ListMove::PageUp, shift),
            KeyCode::End => self.move_index(ListMove::End, shift),
            KeyCode::Home => self.move_index(ListMove::Start, shift),
            KeyCode::Char('a') => {
                if ctrl {
                    self.select_all()
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub const fn reset(&mut self) {
        self.index = 0;
        self.scroll = 0;
        self.selector = None;
    }

    pub fn render<T>(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        items: impl IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
        mut render_line: impl FnMut(Rect, &mut Buffer, T, ListItem),
    ) {
        let items = items.into_iter();

        // Make sure index and selector is not out of bounds
        let max_idx = items.len().saturating_sub(1);
        self.index = self.index.min(max_idx);
        self.selector = self.selector.map(|selector| selector.min(max_idx));

        // Determine scroll
        let scroll = if self.height != area.height {
            // Refresh scroll on window resize
            0
        } else {
            self.scroll
        };
        self.scroll = crate::Scrollbar::calculate_scroll_with_margins(
            items.len(),
            area.height,
            self.index,
            scroll,
            self.margin_top,
            self.margin_bottom,
            self.padding_bottom,
        );

        self.len = items.len();
        self.height = area.height;

        // Scrollbar
        if let Some(colors) = self.scrollbar {
            if Scrollbar::is_scrollable(items.len(), area.as_size()) {
                let scroll_area = Scrollbar::make_scroll_area(&mut area);
                Scrollbar::new().with_colors(colors).render(
                    scroll_area,
                    buf,
                    self.scroll,
                    items.len(),
                );
            }
        }

        // Render
        let selection = self.selection_inclusive();
        let mut line = Rect { height: 1, ..area };

        items
            .enumerate()
            .skip(self.scroll)
            .take(area.height as usize)
            .for_each(|(i, item)| {
                let list_item = if i == self.index {
                    ListItem::Selected
                } else if selection.contains(&i) {
                    ListItem::Selection
                } else {
                    ListItem::Normal
                };

                render_line(line, buf, item, list_item);

                line.y += 1;
            });
    }

    fn set_index_and_selector(&mut self, i: usize, shift: bool) -> bool {
        let old_index = self.index;
        let old_selector = self.selector;

        if shift {
            if self.selector.is_none() {
                self.selector = Some(self.index);
            }
        } else {
            self.selector = None;
        }

        self.index = usize::min(i, self.len.saturating_sub(1));
        self.selector.take_if(|s| *s == self.index);

        old_index != self.index || old_selector != self.selector
    }
}

use std::{cmp::Ordering, ops::Range};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Style},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{CursorDelete, CursorMove};

pub struct TextInput {
    input: String,
    placeholder: &'static str,
    cursor: usize,
    selector: Option<usize>,
    scroll: usize,
    disabled: bool,
    margin_top: usize,
    margin_bottom: usize,
    colors: TextInputColors,
    last_width: u16,
}

impl TextInput {
    pub const fn new() -> Self {
        Self::from(String::new())
    }

    pub const fn from(s: String) -> Self {
        Self {
            input: s,
            placeholder: "",
            cursor: 0,
            selector: None,
            scroll: 0,
            disabled: false,
            margin_top: 0,
            margin_bottom: 0,
            colors: TextInputColors::new(),
            last_width: 0,
        }
    }

    pub const fn with_placeholder(mut self, s: &'static str) -> Self {
        self.placeholder = s;
        self
    }

    pub const fn with_colors(mut self, colors: TextInputColors) -> Self {
        self.set_colors(colors);
        self
    }

    pub const fn with_margins(mut self, top: usize, bottom: usize) -> Self {
        self.margin_top = top;
        self.margin_bottom = bottom;
        self
    }

    pub const fn with_disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub const fn set_colors(&mut self, colors: TextInputColors) -> &mut Self {
        self.colors = colors;
        self
    }

    pub const fn set_disabled(&mut self, value: bool) -> &mut Self {
        self.disabled = value;
        self
    }

    pub const fn set_enabled(&mut self, value: bool) -> &mut Self {
        self.set_disabled(!value)
    }

    pub const fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn is_empty_trim(&self) -> bool {
        self.input.as_str().trim().is_empty()
    }

    pub const fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn as_str_trim(&self) -> &str {
        self.input.as_str().trim()
    }

    pub fn hash(&self) -> u64 {
        utils::hash_fast(self.input.as_str())
    }

    pub fn hash_trim(&self) -> u64 {
        utils::hash_fast(self.input.as_str().trim())
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        if self.disabled {
            return false;
        }

        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => self.move_cursor(CursorMove::Down, shift),
            KeyCode::Home => self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => self.move_cursor(CursorMove::End, shift),
            KeyCode::Backspace => self.delete(CursorDelete::Back),
            KeyCode::Delete => self.delete(CursorDelete::Forward),
            KeyCode::Char('a') => {
                if ctrl {
                    self.select_all()
                } else {
                    self.push_char('a');
                    true
                }
            }
            KeyCode::Char(c) => {
                self.push_char(c);
                true
            }
            _ => false,
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.delete_selection();

        let c = if c.is_whitespace() { ' ' } else { c };
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        self.delete_selection();

        s.graphemes(true)
            .map(|g| {
                if g.chars().any(|c| c.is_whitespace()) {
                    " "
                } else {
                    g
                }
            })
            .for_each(|g| {
                self.input.insert_str(self.cursor, g);
                self.cursor += g.len();
            });
    }

    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) -> bool {
        let (old_cursor, old_selector) = (self.cursor, self.selector);

        if shift {
            if self.selector.is_none() {
                self.selector = Some(self.cursor);
            }
        } else {
            self.selector = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(g) = self.input[self.cursor..].graphemes(true).next() {
                    self.cursor += g.len();
                }
            }
            CursorMove::Back => {
                if let Some(g) = self.input[..self.cursor].graphemes(true).rev().next() {
                    self.cursor -= g.len();
                }
            }
            CursorMove::Up | CursorMove::Start => {
                self.cursor = 0;
            }
            CursorMove::Down | CursorMove::End => {
                self.cursor = self.input.len();
            }
        }

        self.selector.take_if(|s| *s == self.cursor);

        self.cursor != old_cursor || self.selector != old_selector
    }

    pub fn select_all(&mut self) -> bool {
        let (old_cursor, old_selector) = (self.cursor, self.selector);

        self.cursor = self.input.len();
        self.selector = Some(0);

        self.cursor != old_cursor || self.selector != old_selector
    }

    pub fn delete(&mut self, cd: CursorDelete) -> bool {
        match cd {
            CursorDelete::Forward => {
                if self.delete_selection() {
                    return true;
                }

                match self.input[self.cursor..].graphemes(true).next() {
                    Some(g) => {
                        self.input
                            .replace_range(self.cursor..self.cursor + g.len(), "");
                        true
                    }
                    None => false,
                }
            }
            CursorDelete::Back => {
                if self.delete_selection() {
                    return true;
                }

                match self.input[..self.cursor].graphemes(true).rev().next() {
                    Some(g) => {
                        self.cursor -= g.len();
                        self.input
                            .replace_range(self.cursor..self.cursor + g.len(), "");
                        true
                    }
                    None => false,
                }
            }
            CursorDelete::Selection => self.delete_selection(),
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.selector = None;
        self.scroll = 0;
    }

    pub fn render(&mut self, line: Rect, buf: &mut Buffer) {
        if line.is_empty() || buf.cell(line.as_position()).is_none() {
            return;
        }

        if self.disabled {
            let Rect { x, y, .. } = line;
            let s = if self.input.is_empty() {
                self.placeholder
            } else {
                self.input.as_str()
            };
            buf.set_stringn(x, y, s, line.width as usize, self.colors.disabled);
            return;
        }

        let cursor_style = Style::new().fg(self.colors.cursor).reversed();
        let selection_style = Style::new().fg(self.colors.selector).reversed();
        let normal_style = Style::new().fg(self.colors.normal);

        if self.input.is_empty() {
            let Rect { x, y, .. } = line;
            buf.set_stringn(
                x,
                y,
                self.placeholder,
                line.width as usize,
                self.colors.placeholder,
            );
            buf[(x, y)].set_style(cursor_style);
            return;
        }

        // Get total input width
        let total_width = unicode_width::UnicodeWidthStr::width(self.input.as_str());

        // Determine scroll
        let scroll = if self.last_width != line.width {
            // Refresh scroll on window resize
            0
        } else {
            self.scroll
        };
        self.scroll = crate::Scrollbar::calculate_scroll_with_margins(
            total_width + 1,
            line.width,
            self.cursor,
            scroll,
            self.margin_top,
            self.margin_bottom,
            0,
        );
        self.last_width = line.width;

        // Render
        let selection = self.try_selection().unwrap_or(self.cursor..self.cursor);
        let max_width = line.width as usize;
        let mut width = 0;
        let Rect { mut x, y, .. } = line;

        for (i, g) in self.input.grapheme_indices(true) {
            let grapheme_width = unicode_width::UnicodeWidthStr::width(g);
            width += grapheme_width;

            if width > max_width + self.scroll {
                break;
            } else if width > self.scroll {
                let is_cursor = i == self.cursor;
                let is_selected = selection.contains(&i);
                let style = if is_cursor {
                    cursor_style
                } else if is_selected {
                    selection_style
                } else {
                    normal_style
                };
                (x, _) = buf.set_stringn(x, y, g, grapheme_width, style);
            }
        }

        if self.cursor == self.input.len()
            && let Some(cell) = buf.cell_mut((x, y))
        {
            cell.set_style(cursor_style);
        }
    }

    fn selection(&self, selector: usize) -> Option<Range<usize>> {
        match self.cursor.cmp(&selector) {
            Ordering::Less => Some(self.cursor..selector),
            Ordering::Greater => Some(selector..self.cursor),
            Ordering::Equal => None,
        }
    }

    fn try_selection(&self) -> Option<Range<usize>> {
        self.selector.and_then(|selector| self.selection(selector))
    }

    fn take_selection(&mut self) -> Option<Range<usize>> {
        self.selector
            .take()
            .and_then(|selector| self.selection(selector))
    }

    fn delete_selection(&mut self) -> bool {
        let Some(range) = self.take_selection() else {
            return false;
        };
        self.cursor = range.start;
        self.input.replace_range(range, "");
        true
    }
}

pub struct TextInputColors {
    pub normal: Color,
    pub cursor: Color,
    pub selector: Color,
    pub placeholder: Color,
    pub disabled: Color,
}

impl TextInputColors {
    pub const fn new() -> Self {
        Self {
            normal: Color::Reset,
            cursor: Color::White,
            selector: Color::DarkGray,
            placeholder: Color::DarkGray,
            disabled: Color::DarkGray,
        }
    }

    pub const fn all(color: Color) -> Self {
        Self {
            normal: color,
            cursor: color,
            selector: color,
            placeholder: color,
            disabled: color,
        }
    }
}

impl Default for TextInputColors {
    fn default() -> Self {
        Self::new()
    }
}

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use unicode_segmentation::UnicodeSegmentation;
use widgets::Shortcut;

use crate::app::Colors;

pub struct TextEditor {
    input: String,
    placeholder: &'static str,
    line_width: u16,
    cursor_index: usize,
    cursor_column: u16,
    cursor_line_index: usize,
    line_start_indexes: Vec<usize>,
    selection_start: Option<usize>,
    scroll: usize,
    lines: Vec<Line<'static>>,
}

pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Start,
    End,
}

pub enum CursorDelete {
    Forward,
    Back,
    _Selection,
}

impl TextEditor {
    pub const SHORTCUTS: [Shortcut<'static>; 2] =
        [Shortcut::new("Copy", "^c"), Shortcut::new("Paste", "^v")];

    pub const fn new() -> Self {
        Self {
            input: String::new(),
            placeholder: "",
            line_width: 0,
            cursor_index: 0,
            cursor_column: 0,
            cursor_line_index: 0,
            line_start_indexes: Vec::new(),
            selection_start: None,
            scroll: 0,
            lines: Vec::new(),
        }
    }

    pub const fn with_placeholder(mut self, s: &'static str) -> Self {
        self.placeholder = s;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => return self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => return self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => return self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => return self.move_cursor(CursorMove::Down, shift),
            KeyCode::Home => return self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => return self.move_cursor(CursorMove::End, shift),
            KeyCode::Backspace => return self.delete(CursorDelete::Back),
            KeyCode::Delete => return self.delete(CursorDelete::Forward),
            KeyCode::Enter => {
                #[cfg(target_os = "windows")]
                self.push_str("\r\n");
                #[cfg(not(target_os = "windows"))]
                self.push_char('\n');
                return true;
            }
            KeyCode::Char(c) => match c {
                'a' => {
                    if ctrl {
                        return self.select_all();
                    }

                    self.push_char(c);
                    return true;
                }
                'c' => {
                    if ctrl {
                        if let Some(selector) = self.selection_start {
                            if let Some(range) = self.get_selection_range(selector) {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(&self.input[range]);
                                }
                            }
                        }
                    } else {
                        self.push_char(c);
                        return true;
                    }
                }
                'v' => {
                    if ctrl {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(s) = clipboard.get_text() {
                                self.push_str(&s);
                                return true;
                            }
                        }
                    } else {
                        self.push_char(c);
                        return true;
                    }
                }
                _ => {
                    self.push_char(c);
                    return true;
                }
            },
            _ => {}
        }

        false
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        self.input.insert(self.cursor_index, c);
        self.cursor_index += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        self.input.insert_str(self.cursor_index, s);
        self.cursor_index += s.len();
    }

    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) -> bool {
        let (old_cursor, old_selector) = (self.cursor_index, self.selection_start);

        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_index);
            }
        } else {
            self.selection_start = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(g) = self.input[self.cursor_index..].graphemes(true).next() {
                    self.cursor_index += g.len();
                }
            }
            CursorMove::Back => {
                if let Some(g) = self.input[..self.cursor_index].graphemes(true).rev().next() {
                    self.cursor_index -= g.len();
                }
            }
            CursorMove::Up => {
                if self.cursor_line_index == 0 {
                    self.cursor_index = 0;
                } else {
                    self.jump_to_line(self.cursor_line_index - 1);
                }
            }
            CursorMove::Down => {
                if self.cursor_line_index == self.line_start_indexes.len() - 1 {
                    self.cursor_index = self.input.len();
                } else {
                    self.jump_to_line(self.cursor_line_index + 1);
                }
            }
            CursorMove::Start => {
                self.cursor_index = 0;
            }
            CursorMove::End => {
                self.cursor_index = self.input.len();
            }
        }

        self.selection_start.take_if(|s| *s == self.cursor_index);

        self.cursor_index != old_cursor || self.selection_start != old_selector
    }

    pub fn select_all(&mut self) -> bool {
        let (old_cursor, old_selector) = (self.cursor_index, self.selection_start);

        self.cursor_index = self.input.len();
        self.selection_start = Some(0);

        self.cursor_index != old_cursor || self.selection_start != old_selector
    }

    pub fn delete(&mut self, cd: CursorDelete) -> bool {
        match cd {
            CursorDelete::Forward => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => match self.input[self.cursor_index..].graphemes(true).next() {
                    Some(g) => {
                        self.input
                            .replace_range(self.cursor_index..self.cursor_index + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::Back => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => match self.input[..self.cursor_index].graphemes(true).rev().next() {
                    Some(g) => {
                        self.cursor_index -= g.len();
                        self.input
                            .replace_range(self.cursor_index..self.cursor_index + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::_Selection => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => false,
            },
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_index = 0;
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.selection_start = None;
        self.line_width = 0;
        self.line_start_indexes.clear();
        self.scroll = 0;
        self.lines.clear();
    }

    fn get_selection_range(&self, selector: usize) -> Option<std::ops::Range<usize>> {
        match self.cursor_index.cmp(&selector) {
            std::cmp::Ordering::Less => Some(self.cursor_index..selector),
            std::cmp::Ordering::Greater => Some(selector..self.cursor_index),
            std::cmp::Ordering::Equal => None,
        }
    }

    fn delete_selection(&mut self, selector: usize) -> bool {
        let Some(range) = self.get_selection_range(selector) else {
            return false;
        };
        self.cursor_index = range.start;
        self.input.replace_range(range, "");
        true
    }

    fn jump_to_line(&mut self, i: usize) -> bool {
        let old_cursor = self.cursor_index;

        self.cursor_line_index = i;
        self.cursor_index = self.line_start_indexes[i];

        let mut graphemes = self.input[self.cursor_index..].graphemes(true);
        let mut column = 0;
        let mut offset = 0;

        loop {
            if column >= self.cursor_column {
                break;
            }

            let Some(g) = graphemes.next() else {
                break;
            };

            if g.contains('\n') {
                break;
            }

            let grapheme_width = if g.chars().any(|c| c.is_whitespace()) {
                1
            } else {
                unicode_width::UnicodeWidthStr::width(g)
            };

            column += grapheme_width as u16;
            offset += g.len();
        }

        self.cursor_column = column;
        self.cursor_index += offset;

        self.cursor_index != old_cursor
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, colors: &Colors) {
        self.lines.clear();
        self.line_start_indexes.clear();
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.line_width = area.width;

        let mut column = 0;
        let mut line_index = 0;
        let input_len = self.input.len();
        let selection_start = self
            .cursor_index
            .min(self.selection_start.unwrap_or(self.cursor_index));
        let selection_end = self
            .selection_start
            .unwrap_or(self.cursor_index)
            .max(self.cursor_index);
        let cursor_style = Style::new().bg(colors.accent).fg(colors.on_accent);
        let selector_style = cursor_style;

        self.line_start_indexes.push(0);
        self.lines.push(Line::default());

        let mut graphemes = self.input.grapheme_indices(true);

        loop {
            let Some((i, g)) = graphemes.next() else {
                if self.cursor_index == input_len {
                    self.cursor_line_index = line_index;
                    self.cursor_column = column;
                    self.lines[line_index].push_span(Span::styled(" ", cursor_style));
                }
                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i < selection_end;

            let style = if is_cursor {
                self.cursor_line_index = line_index;
                self.cursor_column = column;
                cursor_style
            } else if is_selected {
                selector_style
            } else {
                Style::new()
            };

            let (is_next_line, span) = if g.contains('\n') {
                if is_cursor || (is_selected && i < input_len) {
                    self.lines[line_index].push_span(Span::styled(" ", style));
                }
                (true, None)
            } else if g.chars().any(|c| c.is_whitespace()) {
                self.lines[line_index].push_span(Span::styled(" ", style));
                column += 1;
                (column >= area.width, None)
            } else {
                let span = Span::styled(g.to_string(), style);
                column += span.width() as u16;
                if column > area.width {
                    (true, Some(span))
                } else if column == area.width {
                    self.lines[line_index].push_span(span);
                    (true, None)
                } else {
                    self.lines[line_index].push_span(span);
                    (false, None)
                }
            };

            if is_next_line {
                column = 0;
                line_index += 1;
                self.lines.push(Line::default());
                if let Some(span) = span {
                    column += span.width() as u16;
                    self.lines[line_index].push_span(span);
                    self.line_start_indexes.push(i);
                    if is_cursor {
                        self.cursor_line_index = line_index;
                        self.cursor_column = 0;
                    }
                } else {
                    self.line_start_indexes.push(i + g.len());
                }
            }
        }

        if self.input.is_empty() {
            self.lines[0].push_span(Span::styled(
                self.placeholder,
                Style::new().italic().fg(colors.neutral),
            ));
        }

        let height = area.height as usize;
        if self.cursor_line_index > self.scroll {
            let height_diff = self.cursor_line_index - self.scroll;
            let height = height.saturating_sub(1);
            if height_diff > height {
                self.scroll += height_diff - height;
            }
        } else if self.scroll > self.cursor_line_index {
            let height_diff = self.scroll - self.cursor_line_index;
            self.scroll -= height_diff;
        }

        let mut line_area = area;
        line_area.height = 1;

        self.lines
            .iter()
            .skip(self.scroll)
            .take(height)
            .for_each(|line| {
                line.render(line_area, buf);
                line_area.y += 1;
            });
    }
}

pub struct TextInput {
    input: String,
    placeholder: &'static str,
    cursor_index: usize,
    cursor_column: usize,
    selection_start: Option<usize>,
    scroll: usize,
    spans: Vec<Span<'static>>,
}

impl TextInput {
    pub const SHORTCUTS: [Shortcut<'static>; 2] =
        [Shortcut::new("Copy", "^c"), Shortcut::new("Paste", "^v")];

    pub const fn new() -> Self {
        Self {
            input: String::new(),
            placeholder: "",
            cursor_index: 0,
            cursor_column: 0,
            selection_start: None,
            scroll: 0,
            spans: Vec::new(),
        }
    }

    pub const fn with_placeholder(mut self, s: &'static str) -> Self {
        self.placeholder = s;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => return self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => return self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => return self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => return self.move_cursor(CursorMove::Down, shift),
            KeyCode::Home => return self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => return self.move_cursor(CursorMove::End, shift),
            KeyCode::Backspace => return self.delete(CursorDelete::Back),
            KeyCode::Delete => return self.delete(CursorDelete::Forward),
            KeyCode::Char(c) => match c {
                'a' => {
                    if ctrl {
                        return self.select_all();
                    }

                    self.push_char(c);
                    return true;
                }
                'c' => {
                    if ctrl {
                        if let Some(selector) = self.selection_start {
                            if let Some(range) = self.get_selection_range(selector) {
                                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                    let _ = clipboard.set_text(&self.input[range]);
                                }
                            }
                        }
                    } else {
                        self.push_char(c);
                        return true;
                    }
                }
                'v' => {
                    if ctrl {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(s) = clipboard.get_text() {
                                self.push_str(&s);
                                return true;
                            }
                        }
                    } else {
                        self.push_char(c);
                        return true;
                    }
                }
                _ => {
                    self.push_char(c);
                    return true;
                }
            },
            _ => {}
        }

        false
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        let c = if c.is_whitespace() { ' ' } else { c };
        self.input.insert(self.cursor_index, c);
        self.cursor_index += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(start) = self.selection_start.take() {
            self.delete_selection(start);
        }
        s.graphemes(true)
            .map(|g| {
                if g.chars().any(|c| c.is_whitespace()) {
                    " "
                } else {
                    g
                }
            })
            .for_each(|g| {
                self.input.insert_str(self.cursor_index, g);
                self.cursor_index += g.len();
            });
    }

    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) -> bool {
        let (old_cursor, old_selector) = (self.cursor_index, self.selection_start);

        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor_index);
            }
        } else {
            self.selection_start = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(g) = self.input[self.cursor_index..].graphemes(true).next() {
                    self.cursor_index += g.len();
                }
            }
            CursorMove::Back => {
                if let Some(g) = self.input[..self.cursor_index].graphemes(true).rev().next() {
                    self.cursor_index -= g.len();
                }
            }
            CursorMove::Up | CursorMove::Start => {
                self.cursor_index = 0;
            }
            CursorMove::Down | CursorMove::End => {
                self.cursor_index = self.input.len();
            }
        }

        self.selection_start.take_if(|s| *s == self.cursor_index);

        self.cursor_index != old_cursor || self.selection_start != old_selector
    }

    pub fn select_all(&mut self) -> bool {
        let (old_cursor, old_selector) = (self.cursor_index, self.selection_start);

        self.cursor_index = self.input.len();
        self.selection_start = Some(0);

        self.cursor_index != old_cursor || self.selection_start != old_selector
    }

    pub fn delete(&mut self, cd: CursorDelete) -> bool {
        match cd {
            CursorDelete::Forward => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => match self.input[self.cursor_index..].graphemes(true).next() {
                    Some(g) => {
                        self.input
                            .replace_range(self.cursor_index..self.cursor_index + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::Back => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => match self.input[..self.cursor_index].graphemes(true).rev().next() {
                    Some(g) => {
                        self.cursor_index -= g.len();
                        self.input
                            .replace_range(self.cursor_index..self.cursor_index + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::_Selection => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => false,
            },
        }
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_index = 0;
        self.cursor_column = 0;
        self.selection_start = None;
        self.scroll = 0;
        self.spans.clear();
    }

    fn get_selection_range(&self, selector: usize) -> Option<std::ops::Range<usize>> {
        match self.cursor_index.cmp(&selector) {
            std::cmp::Ordering::Less => Some(self.cursor_index..selector),
            std::cmp::Ordering::Greater => Some(selector..self.cursor_index),
            std::cmp::Ordering::Equal => None,
        }
    }

    fn delete_selection(&mut self, selector: usize) -> bool {
        let Some(range) = self.get_selection_range(selector) else {
            return false;
        };
        self.cursor_index = range.start;
        self.input.replace_range(range, "");
        true
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, colors: &Colors) {
        self.spans.clear();
        self.cursor_column = 0;

        let mut total_width = 0;
        let input_len = self.input.len();
        let selection_start = self
            .cursor_index
            .min(self.selection_start.unwrap_or(self.cursor_index));
        let selection_end = self
            .selection_start
            .unwrap_or(self.cursor_index)
            .max(self.cursor_index);
        let cursor_style = Style::new().bg(colors.accent).fg(colors.on_accent);
        let selector_style = cursor_style;

        let mut graphemes = self.input.grapheme_indices(true);

        loop {
            let Some((i, g)) = graphemes.next() else {
                if self.cursor_index == input_len {
                    self.cursor_column = total_width;
                    self.spans.push(Span::styled(" ", cursor_style));
                }
                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i < selection_end;

            let style = if is_cursor {
                self.cursor_column = total_width;
                cursor_style
            } else if is_selected {
                selector_style
            } else {
                Style::new()
            };

            let span = Span::styled(g.to_string(), style);
            total_width += span.width();
            self.spans.push(span);
        }

        if self.input.is_empty() {
            self.spans.push(Span::styled(
                self.placeholder,
                Style::new().italic().fg(colors.neutral),
            ));
        }

        // todo: fix scroll when left-most char has width > 1
        let line_width = area.width as usize;
        if self.cursor_column > self.scroll {
            let width_diff = self.cursor_column - self.scroll;
            let line_width = line_width.saturating_sub(1);
            if width_diff > line_width {
                self.scroll += width_diff - line_width;
            }
        } else if self.scroll > self.cursor_column {
            let width_diff = self.scroll - self.cursor_column;
            self.scroll -= width_diff;
        }

        let mut skip_width = 0;
        let mut input_width = 0;
        let mut span_area = Rect { height: 1, ..area };

        for span in self.spans.iter() {
            let span_width = span.width();
            skip_width += span_width;
            if skip_width > self.scroll && input_width < line_width {
                input_width += span_width;
                span_area.width = span_width as u16;
                span.render(span_area, buf);
                span_area.x += span_width as u16;
            }
        }
    }
}

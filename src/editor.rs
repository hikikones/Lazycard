use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::utils::{STYLE_CURSOR, STYLE_LABEL, STYLE_NONE, STYLE_SELECTED};

pub struct TextEditor {
    input: String,
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
    Selection,
}

impl TextEditor {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
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

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => self.move_cursor(CursorMove::Down, shift),
            KeyCode::Backspace => self.delete(CursorDelete::Back),
            KeyCode::Delete => self.delete(CursorDelete::Forward),
            KeyCode::Home => self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => self.move_cursor(CursorMove::End, shift),
            KeyCode::Enter => {
                self.push_char('\n');
                true
            }
            KeyCode::Char(c) => match c {
                'a' => {
                    if ctrl {
                        self.select_all()
                    } else {
                        self.push_char(c);
                        true
                    }
                }
                _ => {
                    self.push_char(c);
                    true
                }
            },
            _ => false,
        }
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
                if let Some(c) = self.input[self.cursor_index..].chars().next() {
                    self.cursor_index += c.len_utf8();
                }
            }
            CursorMove::Back => {
                if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                    self.cursor_index -= c.len_utf8();
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
                None => {
                    if self.input[self.cursor_index..].chars().next().is_some() {
                        self.input.remove(self.cursor_index);
                        true
                    } else {
                        false
                    }
                }
            },
            CursorDelete::Back => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => {
                    if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                        self.cursor_index -= c.len_utf8();
                        self.input.remove(self.cursor_index);
                        true
                    } else {
                        false
                    }
                }
            },
            CursorDelete::Selection => {
                if let Some(selector) = self.selection_start.take() {
                    self.delete_selection(selector)
                } else {
                    false
                }
            }
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
        self.lines.clear();
        self.scroll = 0;
    }

    fn delete_selection(&mut self, selector: usize) -> bool {
        let (start, end) = if self.cursor_index < selector {
            (self.cursor_index, selector)
        } else {
            (selector, self.cursor_index)
        };
        self.input.replace_range(start..end, "");
        self.cursor_index = start;
        start != end
    }

    fn jump_to_line(&mut self, i: usize) -> bool {
        let old_cursor = self.cursor_index;

        self.cursor_line_index = i;
        self.cursor_index = self.line_start_indexes[i];

        let mut chars = self.input[self.cursor_index..].chars();
        let mut column = 0;
        let mut offset = 0;

        loop {
            if column >= self.cursor_column {
                break;
            }

            let Some(c) = chars.next() else {
                break;
            };

            if c == '\n' {
                break;
            }

            let char_width = if c.is_whitespace() {
                1
            } else {
                unicode_width::UnicodeWidthChar::width(c).unwrap_or(1) as u16
            };

            column += char_width;
            offset += c.len_utf8();
        }

        self.cursor_column = column;
        self.cursor_index += offset;

        self.cursor_index != old_cursor
    }
}

impl Widget for &mut TextEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.lines.clear();
        self.line_start_indexes.clear();
        self.cursor_column = 0;
        self.cursor_line_index = 0;
        self.line_width = area.width;

        let mut line_width = 0;
        let mut line_index = 0;
        let input_len = self.input.len();
        let selection_start = self
            .cursor_index
            .min(self.selection_start.unwrap_or(self.cursor_index));
        let selection_end = self
            .selection_start
            .unwrap_or(self.cursor_index)
            .max(self.cursor_index);

        self.line_start_indexes.push(0);
        self.lines.push(Line::default());

        let mut chars = self.input.char_indices();

        loop {
            let Some((i, c)) = chars.next() else {
                if self.cursor_index == input_len {
                    self.cursor_line_index = line_index;
                    self.cursor_column = line_width;
                    self.lines[line_index].push_span(Span::styled(" ", STYLE_CURSOR));
                }
                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i < selection_end;

            let style = if is_cursor {
                self.cursor_line_index = line_index;
                self.cursor_column = line_width;
                STYLE_CURSOR
            } else if is_selected {
                STYLE_SELECTED
            } else {
                STYLE_NONE
            };

            let (is_next_line, span) = if c == '\n' {
                if is_cursor || (is_selected && i < input_len) {
                    self.lines[line_index].push_span(Span::styled(" ", style));
                }
                (true, None)
            } else if c.is_whitespace() {
                self.lines[line_index].push_span(Span::styled(" ", style));
                line_width += 1;
                (line_width >= area.width, None)
            } else {
                let span = Span::styled(c.to_string(), style);
                line_width += span.width() as u16;
                if line_width > area.width {
                    (true, Some(span))
                } else {
                    self.lines[line_index].push_span(span);
                    (false, None)
                }
            };

            if is_next_line {
                line_width = 0;
                line_index += 1;
                self.lines.push(Line::default());
                if let Some(span) = span {
                    line_width += span.width() as u16;
                    self.lines[line_index].push_span(span);
                    self.line_start_indexes.push(i);
                } else {
                    self.line_start_indexes.push(i + c.len_utf8());
                }
            }
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
                line.render_ref(line_area, buf);
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

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => self.move_cursor(CursorMove::Down, shift),
            KeyCode::Backspace => self.delete(CursorDelete::Back),
            KeyCode::Delete => self.delete(CursorDelete::Forward),
            KeyCode::Home => self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => self.move_cursor(CursorMove::End, shift),
            KeyCode::Enter => {
                self.push_char('\n');
                true
            }
            KeyCode::Char(c) => match c {
                'a' => {
                    if ctrl {
                        self.select_all()
                    } else {
                        self.push_char(c);
                        true
                    }
                }
                _ => {
                    self.push_char(c);
                    true
                }
            },
            _ => false,
        }
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
                if let Some(c) = self.input[self.cursor_index..].chars().next() {
                    self.cursor_index += c.len_utf8();
                }
            }
            CursorMove::Back => {
                if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                    self.cursor_index -= c.len_utf8();
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
                None => {
                    if self.input[self.cursor_index..].chars().next().is_some() {
                        self.input.remove(self.cursor_index);
                        true
                    } else {
                        false
                    }
                }
            },
            CursorDelete::Back => match self.selection_start.take() {
                Some(selector) => self.delete_selection(selector),
                None => {
                    if let Some(c) = self.input[..self.cursor_index].chars().rev().next() {
                        self.cursor_index -= c.len_utf8();
                        self.input.remove(self.cursor_index);
                        true
                    } else {
                        false
                    }
                }
            },
            CursorDelete::Selection => {
                if let Some(selector) = self.selection_start.take() {
                    self.delete_selection(selector)
                } else {
                    false
                }
            }
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

    fn delete_selection(&mut self, selector: usize) -> bool {
        let (start, end) = if self.cursor_index < selector {
            (self.cursor_index, selector)
        } else {
            (selector, self.cursor_index)
        };
        self.input.replace_range(start..end, "");
        self.cursor_index = start;
        start != end
    }
}

impl Widget for &mut TextInput {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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

        let mut chars =
            self.input.char_indices().map(
                |(i, c)| {
                    if c.is_whitespace() {
                        (i, ' ')
                    } else {
                        (i, c)
                    }
                },
            );

        loop {
            let Some((i, c)) = chars.next() else {
                if self.cursor_index == input_len {
                    self.cursor_column = total_width;
                    self.spans.push(Span::styled(" ", STYLE_CURSOR));
                }
                break;
            };

            let is_cursor = i == self.cursor_index;
            let is_selected = i >= selection_start && i < selection_end;

            let style = if is_cursor {
                self.cursor_column = total_width;
                STYLE_CURSOR
            } else if is_selected {
                STYLE_SELECTED
            } else {
                STYLE_NONE
            };

            let span = Span::styled(c.to_string(), style);
            total_width += span.width();
            self.spans.push(span);
        }

        if self.input.is_empty() {
            self.spans.push(Span::raw(" "));
            self.spans.extend(
                self.placeholder.chars().map(|c| {
                    Span::styled(c.to_string(), STYLE_LABEL.add_modifier(Modifier::ITALIC))
                }),
            );
        }

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
                span.render_ref(span_area, buf);
                span_area.x += span_width as u16;
            }
        }
    }
}

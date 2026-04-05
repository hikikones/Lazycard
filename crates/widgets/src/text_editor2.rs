use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::CursorMove;

pub struct TextEditor2 {
    input: String,
    cursor: usize,
    lines: String,
    cursor_offset: usize,
}

impl TextEditor2 {
    pub fn new() -> Self {
        Self {
            input: String::from(
                "this is test text\na new line comes here\nand another one\nlast one that is very long and should wrap around at the end because why not huh this is getting     tedious      why not some more text or ok done\n",
            ),
            cursor: 0,
            lines: String::new(),
            cursor_offset: 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub const fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn push_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        self.input.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    pub fn move_cursor(&mut self, cm: CursorMove) -> bool {
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
            CursorMove::Up => {
                todo!()
            }
            CursorMove::Down => {
                todo!()
            }
            CursorMove::Start => {
                self.cursor = 0;
            }
            CursorMove::End => {
                self.cursor = self.input.len();
            }
        }

        true
    }

    pub fn input(&mut self, key_pressed: KeyCode, _key_modifiers: KeyModifiers) -> bool {
        match key_pressed {
            KeyCode::Right => return self.move_cursor(CursorMove::Forward),
            KeyCode::Left => return self.move_cursor(CursorMove::Back),
            KeyCode::Up => return self.move_cursor(CursorMove::Up),
            KeyCode::Down => return self.move_cursor(CursorMove::Down),
            KeyCode::Home => return self.move_cursor(CursorMove::Start),
            KeyCode::End => return self.move_cursor(CursorMove::End),
            KeyCode::Enter => {
                self.push_char('\n');
            }
            _ => {}
        }

        true
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.lines.clear();
        self.cursor_offset = 0;

        let line_width = area.width as usize;
        // let is_cursor = i == self.cursor;
        // buf.set_string(area.x, area.y, self.as_str(), Style::new());

        let mut curr_width = 0;
        let mut word_start = None;
        let mut graphemes = self.input.grapheme_indices(true);
        loop {
            let Some((i, g)) = graphemes.next() else {
                break;
            };

            let cursor = self.cursor + self.cursor_offset;
            let is_cursor = i == cursor;
            let is_before_cursor = i < cursor;
            // let gw = unicode_width::UnicodeWidthStr::width(g);

            if g.contains('\n') {
                // Take word
                if let Some(start) = word_start.take() {
                    let word = &self.input[start..i];
                    let word_width = unicode_width::UnicodeWidthStr::width(word);
                    if curr_width + word_width > line_width {
                        curr_width = word_width;
                        self.lines.push('\n');
                        self.lines.push_str(word);
                        if is_before_cursor {
                            self.cursor_offset += 1;
                        }
                    } else {
                        curr_width += word_width;
                        self.lines.push_str(word);
                    }
                }
                curr_width = 0;
                // word_start = None;
                self.lines.push_str(g);
            } else if g.chars().any(|c| c.is_whitespace()) {
                // Take word
                if let Some(start) = word_start.take() {
                    let word = &self.input[start..i];
                    let word_width = unicode_width::UnicodeWidthStr::width(word);
                    if curr_width + word_width > line_width {
                        curr_width = word_width;
                        self.lines.push('\n');
                        self.lines.push_str(word);
                        if is_before_cursor {
                            self.cursor_offset += 1;
                        }
                    } else {
                        curr_width += word_width;
                        self.lines.push_str(word);
                    }
                }

                let gw = unicode_width::UnicodeWidthStr::width(g);
                if curr_width + gw > line_width {
                    curr_width = 0;
                    self.lines.push('\n');
                    if is_before_cursor {
                        self.cursor_offset += 1;
                    }
                }
                curr_width += gw;
                self.lines.push_str(g);
            } else {
                //Build word
                if word_start.is_none() {
                    word_start = Some(i);
                }
            }
        }

        // Take word
        if let Some(start) = word_start.take() {
            let word = &self.input[start..];
            let word_width = unicode_width::UnicodeWidthStr::width(word);
            if curr_width + word_width > line_width {
                curr_width = word_width;
                self.lines.push('\n');
                self.lines.push_str(word);
                // if is_before_cursor {
                //     self.cursor_offset += 1;
                // }
            } else {
                curr_width += word_width;
                self.lines.push_str(word);
            }
        }

        // for (i, s) in self.lines.lines().enumerate() {
        //     buf.set_stringn(area.x, area.y + i as u16, s, line_width, Style::new());
        // }

        let cursor = self.cursor + self.cursor_offset;
        // let cursor = self.cursor;
        let Rect { mut x, mut y, .. } = area;
        for (i, g) in self.lines.grapheme_indices(true) {
            let is_cursor = i == cursor;

            let style = if is_cursor {
                Style::new().reversed()
            } else {
                Style::new()
            };

            if g.contains('\n') {
                buf[(x, y)].set_style(style);
                x = area.x;
                y += 1;

                if y >= area.y + area.height {
                    break;
                } else {
                    continue;
                }
            }

            // (x, _) = buf.set_stringn(x, y, g, usize::MAX, style);
            buf[(x, y)].set_symbol(g).set_style(style);
            x += unicode_width::UnicodeWidthStr::width(g) as u16;
        }

        if self.cursor == self.input.len() {
            buf[(x, y)].set_style(Style::new().reversed());
        }
    }

    fn take_word(
        &mut self,
        word_start: Option<usize>,
        current_index: usize,
        line_width: usize,
        mut current_width: usize,
        is_before_cursor: bool,
    ) -> usize {
        let Some(start) = word_start else {
            return current_width;
        };

        let word = &self.input[start..current_index];
        let word_width = unicode_width::UnicodeWidthStr::width(word);
        if current_width + word_width > line_width {
            current_width = word_width;
            self.lines.push('\n');
            self.lines.push_str(word);
            if is_before_cursor {
                self.cursor_offset += 1;
            }
        } else {
            current_width += word_width;
            self.lines.push_str(word);
        }

        current_width
    }
}

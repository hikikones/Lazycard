use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
};

pub struct TextSegment {
    text: String,
    segments: Vec<(usize, Style)>,
    alignment: Alignment,
    total_width: usize,
}

impl TextSegment {
    pub const fn new() -> Self {
        Self {
            text: String::new(),
            segments: Vec::new(),
            alignment: Alignment::Left,
            total_width: 0,
        }
    }

    pub const fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub const fn set_alignment(&mut self, alignment: Alignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    pub const fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub const fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub const fn width(&self) -> u16 {
        self.total_width as u16
    }

    pub fn push_char(&mut self, ch: char, style: impl Into<Style>) {
        self.text.push(ch);
        self.segments.push((ch.len_utf8(), style.into()));
        self.total_width += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
    }

    pub fn push_chars(&mut self, chars: &[char], style: impl Into<Style>) {
        let start = self.text.len();
        self.text.extend(chars);
        self.push_segment(start, style);
    }

    pub fn repeat_char(&mut self, ch: char, n: usize, style: impl Into<Style>) {
        let start = self.text.len();
        self.text.extend(std::iter::repeat_n(ch, n));

        if self.text.len() == start {
            return;
        }

        self.segments.push((n * ch.len_utf8(), style.into()));
        self.total_width += n * unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
    }

    pub fn push_str(&mut self, text: &str, style: impl Into<Style>) {
        if text.is_empty() {
            return;
        }

        self.text.push_str(text);
        self.segments.push((text.len(), style.into()));
        self.total_width += unicode_width::UnicodeWidthStr::width(text);
    }

    pub fn push_str_iter<'a>(
        &mut self,
        slices: impl IntoIterator<Item = &'a str>,
        style: impl Into<Style>,
    ) {
        let start = self.text.len();
        self.text.extend(slices);
        self.push_segment(start, style);
    }

    pub fn push_int(&mut self, int: impl itoa::Integer, style: impl Into<Style>) {
        let mut buffer = itoa::Buffer::new();
        self.push_str(buffer.format(int), style);
    }

    pub fn push_fmt(&mut self, args: std::fmt::Arguments<'_>, style: impl Into<Style>) {
        use std::fmt::Write;
        let start = self.text.len();
        let _ = self.text.write_fmt(args);
        self.push_segment(start, style);
    }

    pub fn extend<'a>(&mut self, items: impl IntoIterator<Item = (&'a str, Style)>) {
        for (text, style) in items.into_iter() {
            self.push_str(text, style);
        }
    }

    pub fn pop(&mut self) {
        if let Some((len, _)) = self.segments.pop() {
            let start = self.text.len() - len;
            let slice = &self.text[start..];

            self.total_width -= unicode_width::UnicodeWidthStr::width(slice);
            self.text.truncate(start);
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.segments.clear();
        self.total_width = 0;
    }

    pub fn render(&self, line: Rect, buf: &mut Buffer) {
        if buf.cell((line.x, line.y)).is_none() {
            return;
        }

        let line = match self.alignment {
            Alignment::Left => line,
            Alignment::Center => Rect {
                x: line.x + (line.width.saturating_sub(self.width())) / 2,
                ..line
            },
            Alignment::Right => Rect {
                x: line.x + line.width.saturating_sub(self.width()),
                ..line
            },
        };
        let mut start = 0;
        let Rect {
            mut x,
            y,
            mut width,
            ..
        } = line;

        for (len, style) in self.segments.iter().copied() {
            if width == 0 {
                break;
            }

            let end = start + len;
            let text = &self.text[start..end];

            let (next_x, _) = buf.set_stringn(x, y, text, width as usize, style);
            width -= next_x - x;
            x = next_x;
            start = end;
        }
    }

    fn push_segment(&mut self, start: usize, style: impl Into<Style>) {
        if start >= self.text.len() {
            return;
        }

        let text = &self.text[start..];
        self.segments.push((text.len(), style.into()));
        self.total_width += unicode_width::UnicodeWidthStr::width(text);
    }
}

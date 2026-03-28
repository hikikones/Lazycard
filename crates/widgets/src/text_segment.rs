use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
};

pub struct TextSegment {
    text: String,
    segments: Vec<(usize, Style)>,
    total_width: usize,
    alignment: Alignment,
}

impl TextSegment {
    pub const fn new() -> Self {
        Self {
            text: String::new(),
            segments: Vec::new(),
            total_width: 0,
            alignment: Alignment::Left,
        }
    }

    pub const fn with_alignment(mut self, alignment: Alignment) -> Self {
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
        let mut len = 0;
        let mut width = 0;

        for ch in chars.iter().copied() {
            len += ch.len_utf8();
            width += unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            self.text.push(ch);
        }

        self.segments.push((len, style.into()));
        self.total_width += width;
    }

    pub fn repeat_char(&mut self, ch: char, n: usize, style: impl Into<Style>) {
        if n == 0 {
            return;
        }

        for _ in 0..n {
            self.text.push(ch);
        }

        let width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        let len = ch.len_utf8() * n;
        self.segments.push((len, style.into()));
        self.total_width += width * n;
    }

    pub fn push_str(&mut self, text: &str, style: impl Into<Style>) {
        if text.is_empty() {
            return;
        }

        self.text.push_str(text);
        self.segments.push((text.len(), style.into()));
        self.total_width += unicode_width::UnicodeWidthStr::width(text);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = (impl AsRef<str>, impl Into<Style>)>) {
        for (text, style) in items.into_iter() {
            self.push_str(text.as_ref(), style);
        }
    }

    pub fn extend_as_one(
        &mut self,
        slices: impl IntoIterator<Item = impl AsRef<str>>,
        style: impl Into<Style>,
    ) {
        let mut len = 0;
        let mut width = 0;

        for text in slices.into_iter() {
            let text = text.as_ref();
            if text.is_empty() {
                continue;
            }

            len += text.len();
            width += unicode_width::UnicodeWidthStr::width(text);
            self.text.push_str(text);
        }

        self.segments.push((len, style.into()));
        self.total_width += width;
    }

    pub fn pop(&mut self) {
        if let Some((i, _)) = self.segments.pop() {
            let start = self.text.len() - i;
            let end = self.text.len();
            let slice = &self.text[start..end];

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
}

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::Widget,
};

pub struct Shortcut<'a> {
    name: &'a str,
    key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }
}

pub struct Shortcuts<'a> {
    shortcuts: Vec<(ShortcutLine, Shortcut<'a>)>,
    top: Line<'a>,
    middle: Line<'a>,
    bottom: Line<'a>,
    name_color: Color,
    key_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub enum ShortcutLine {
    Top,
    Middle,
    Bottom,
}

impl<'a> Shortcuts<'a> {
    pub fn new() -> Self {
        Self {
            shortcuts: Vec::new(),
            top: Line::default().centered(),
            middle: Line::default().centered(),
            bottom: Line::default().centered(),
            name_color: Color::Reset,
            key_color: Color::Indexed(240),
        }
    }

    pub const fn with_colors(mut self, name: Color, key: Color) -> Self {
        self.name_color = name;
        self.key_color = key;
        self
    }

    pub fn push(&mut self, line: ShortcutLine, shortcut: Shortcut<'a>) {
        self.shortcuts.push((line, shortcut));
    }

    pub fn extend(
        &mut self,
        line: ShortcutLine,
        shortcuts: impl IntoIterator<Item = Shortcut<'a>>,
    ) {
        self.shortcuts
            .extend(shortcuts.into_iter().map(|s| (line, s)));
    }

    pub fn render(&mut self, mut area: Rect, buf: &mut Buffer) {
        for (line, shortcut) in self.shortcuts.drain(..) {
            let spans = [
                Span::raw(" "),
                Span::styled(shortcut.key, self.key_color),
                Span::raw(" "),
                Span::styled(shortcut.name, self.name_color),
                Span::raw(" "),
            ];
            match line {
                ShortcutLine::Top => self.top.extend(spans),
                ShortcutLine::Middle => self.middle.extend(spans),
                ShortcutLine::Bottom => self.bottom.extend(spans),
            }
        }

        (&self.top).render(area, buf);
        area.y += 1;
        (&self.middle).render(area, buf);
        area.y += 1;
        (&self.bottom).render(area, buf);

        self.top.spans.clear();
        self.middle.spans.clear();
        self.bottom.spans.clear();
    }
}

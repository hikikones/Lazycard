use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
};

pub const STYLE_NONE: Style = Style::new();
pub const STYLE_BOLD: Style = Style::new().add_modifier(Modifier::BOLD);
pub const STYLE_ITALIC: Style = Style::new().add_modifier(Modifier::ITALIC);

pub struct Shortcut<'a> {
    name: &'a str,
    key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }

    pub fn as_spans(self, accent_color: Color) -> [Span<'a>; 5] {
        [
            Span::raw(" "),
            Span::styled(self.key, accent_color),
            Span::raw(" "),
            Span::raw(self.name),
            Span::raw(" "),
        ]
    }
}

pub struct Shortcuts<'a> {
    first: Vec<Shortcut<'a>>,
    second: Vec<Shortcut<'a>>,
}

impl<'a> Shortcuts<'a> {
    pub const fn new() -> Self {
        Self {
            first: Vec::new(),
            second: Vec::new(),
        }
    }

    pub fn push_first(&mut self, shortcut: Shortcut<'a>) {
        self.first.push(shortcut);
    }

    pub fn push_second(&mut self, shortcut: Shortcut<'a>) {
        self.second.push(shortcut);
    }

    pub fn extend_first(&mut self, iter: impl IntoIterator<Item = Shortcut<'a>>) {
        self.first.extend(iter);
    }

    pub fn extend_second(&mut self, iter: impl IntoIterator<Item = Shortcut<'a>>) {
        self.second.extend(iter);
    }

    pub fn drain_first(&mut self) -> impl Iterator<Item = Shortcut<'a>> + '_ {
        self.first.drain(..)
    }

    pub fn drain_second(&mut self) -> impl Iterator<Item = Shortcut<'a>> + '_ {
        self.second.drain(..)
    }
}

pub fn _layout_center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    _layout_center_vertical(layout_center_horizontal(area, horizontal), vertical)
}

pub fn layout_center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn _layout_center_vertical(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MatchScore(u32);

impl MatchScore {
    pub const fn new(score: u32) -> Self {
        Self(score)
    }
}

pub struct Matcher {
    matcher: nucleo_matcher::Matcher,
    pattern: nucleo_matcher::pattern::Pattern,
    buffer: Vec<char>,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
            pattern: nucleo_matcher::pattern::Pattern::new(
                "",
                nucleo_matcher::pattern::CaseMatching::Smart,
                nucleo_matcher::pattern::Normalization::Smart,
                nucleo_matcher::pattern::AtomKind::Fuzzy,
            ),
            buffer: Vec::new(),
        }
    }

    pub fn update(&mut self, pattern: &str) {
        self.pattern.reparse(
            pattern,
            nucleo_matcher::pattern::CaseMatching::Smart,
            nucleo_matcher::pattern::Normalization::Smart,
        );
    }

    pub fn score(&mut self, haystack: &str) -> Option<u32> {
        self.pattern.score(
            nucleo_matcher::Utf32Str::new(haystack, &mut self.buffer),
            &mut self.matcher,
        )
    }
}

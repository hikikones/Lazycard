use std::{
    borrow::BorrowMut,
    hash::{DefaultHasher, Hash, Hasher},
    iter::Peekable,
    ops::Range,
    str::CharIndices,
    sync::LazyLock,
};

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::WidgetRef};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    app::Colors,
    utils::{STYLE_BOLD, STYLE_ITALIC, STYLE_NONE},
};

#[derive(Debug)]
pub struct Markup {
    width: usize,
    height: usize,
    hash: u64,
    scroll: usize,
    desired_scroll: Option<usize>,
    lines: Vec<Line<'static>>,
    word_buffer: Vec<Span<'static>>,
}

pub enum ScrollMove {
    Up(usize),
    Down(usize),
    Start,
    End,
}

impl Markup {
    pub const fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            hash: 0,
            scroll: 0,
            desired_scroll: None,
            lines: Vec::new(),
            word_buffer: Vec::new(),
        }
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        let _ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let _shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Down => self.scroll(ScrollMove::Down(1)),
            KeyCode::Up => self.scroll(ScrollMove::Up(1)),
            KeyCode::Home => self.scroll(ScrollMove::Start),
            KeyCode::End => self.scroll(ScrollMove::End),
            _ => false,
        }
    }

    pub fn scroll(&mut self, sm: ScrollMove) -> bool {
        let lines = self.lines.len();
        let height = self.height;
        let old_scroll = self.scroll;

        self.scroll = match sm {
            ScrollMove::Up(n) => calculate_scroll(self.scroll.saturating_sub(n), lines, height),
            ScrollMove::Down(n) => calculate_scroll(self.scroll.saturating_add(n), lines, height),
            ScrollMove::Start => 0,
            ScrollMove::End => calculate_scroll(usize::MAX, lines, height),
        };

        self.scroll != old_scroll
    }

    pub fn desired_scroll(&mut self, sm: ScrollMove) {
        match sm {
            ScrollMove::Up(n) => match self.desired_scroll.as_mut() {
                Some(scroll) => *scroll = scroll.saturating_sub(n),
                None => self.desired_scroll = Some(self.scroll.saturating_sub(n)),
            },
            ScrollMove::Down(n) => match self.desired_scroll.as_mut() {
                Some(scroll) => *scroll = scroll.saturating_add(n),
                None => self.desired_scroll = Some(self.scroll.saturating_add(n)),
            },
            ScrollMove::Start => self.desired_scroll = Some(0),
            ScrollMove::End => self.desired_scroll = Some(usize::MAX),
        }
    }

    pub fn render(&mut self, text: &str, area: Rect, buf: &mut Buffer, colors: &Colors) {
        let width = area.width as usize;
        self.height = area.height as usize;

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        if self.width != width || self.hash != hash {
            self.lines.clear();
            self.width = width;
            self.hash = hash;

            // Process markup
            for (block, _) in BlockParser::new(text) {
                match block {
                    BlockElement::Paragraph { alignment, text } => {
                        self.parse_text(text, alignment, "", "");
                    }
                    BlockElement::Code { language, text } => {
                        self.parse_code(language, text, colors);
                    }
                    BlockElement::List { items } => {
                        for item in items {
                            self.parse_text(item, Alignment::Left, " • ", "   ");
                        }
                    }
                    BlockElement::Break => self.lines.push(
                        Line::styled("——————————", STYLE_NONE.fg(colors.neutral))
                            .alignment(Alignment::Center),
                    ),
                }
                self.lines.push(Line::default());
            }
            self.lines.pop();
        }

        // Update scroll
        let scroll = self.desired_scroll.take().unwrap_or(self.scroll);
        self.scroll = calculate_scroll(scroll, self.lines.len(), self.height);

        // Render lines
        let mut line_area = Rect { height: 1, ..area };
        self.lines
            .iter()
            .skip(self.scroll)
            .take(self.height)
            .for_each(|line| {
                line.render_ref(line_area, buf);
                line_area.y += 1;
            });
    }

    pub fn clear(&mut self) {
        self.width = 0;
        self.height = 0;
        self.hash = 0;
        self.scroll = 0;
        self.desired_scroll = None;
        self.lines.clear();
    }

    fn parse_text(
        &mut self,
        text: &str,
        alignment: Alignment,
        first_indent: &'static str,
        wrap_indent: &'static str,
    ) {
        fn new_line(indent: &'static str, alignment: Alignment) -> (Line<'static>, usize) {
            let mut line = Line::default().alignment(alignment);
            let indent_span = Span::raw(indent);
            let indent_width = indent_span.width();
            line.push_span(indent_span);
            (line, indent_width)
        }

        let width = self.width;
        let (mut line, mut column) = new_line(first_indent, alignment);
        let mut word_width = 0;

        for (tag, span) in InlineParser::new(text) {
            let style = match tag {
                InlineTag::Normal => STYLE_NONE,
                InlineTag::Bold => STYLE_BOLD,
                InlineTag::Italic => STYLE_ITALIC,
            };
            for g in span.graphemes(true) {
                if g.chars().any(|c| c.is_whitespace()) {
                    if self.word_buffer.is_empty() {
                        if column + 1 > width {
                            self.lines.push(line);
                            (line, column) = new_line(wrap_indent, alignment);
                        } else {
                            line.push_span(Span::styled(" ", style));
                            column += 1;
                        }
                    } else {
                        // todo: break word when word_width > width
                        if column + word_width > width {
                            self.lines.push(line);
                            (line, column) = new_line(wrap_indent, alignment);
                        }

                        line.extend(self.word_buffer.drain(..));
                        line.push_span(Span::styled(" ", style));
                        column += word_width + 1;
                    }
                    word_width = 0;
                } else {
                    let g_span = Span::styled(g.to_string(), style);
                    word_width += g_span.width();
                    self.word_buffer.push(g_span);
                }
            }
        }

        if !self.word_buffer.is_empty() {
            // todo: break word when word_width > width
            if column + word_width > width {
                self.lines.push(line);
                (line, _) = new_line(wrap_indent, alignment);
            }

            line.extend(self.word_buffer.drain(..));
        }

        self.lines.push(line);
    }

    fn parse_code(&mut self, language: &str, text: &str, colors: &Colors) {
        static SYNTAX_SET: LazyLock<SyntaxSet> =
            LazyLock::new(|| SyntaxSet::load_defaults_newlines());
        static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| ThemeSet::load_defaults());

        let syntax = if language.is_empty() {
            SYNTAX_SET.find_syntax_plain_text()
        } else {
            SYNTAX_SET
                .find_syntax_by_token(language)
                .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
        };
        let mut highlighter =
            HighlightLines::new(syntax, &THEME_SET.themes[colors.syntax_highlighting]);

        for code_line in LinesWithEndings::from(text.replace('\t', "    ").as_str()) {
            match highlighter.highlight_line(code_line, &SYNTAX_SET) {
                Ok(spans) => {
                    let mut line = Line::default();
                    for (style, span) in spans {
                        let mut modifiers = Modifier::empty();
                        if style.font_style.contains(FontStyle::BOLD) {
                            modifiers.insert(Modifier::BOLD);
                        }
                        if style.font_style.contains(FontStyle::ITALIC) {
                            modifiers.insert(Modifier::ITALIC);
                        }
                        if style.font_style.contains(FontStyle::UNDERLINE) {
                            modifiers.insert(Modifier::UNDERLINED);
                        }
                        let fg =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        line.push_span(Span::styled(
                            span.to_owned(),
                            Style::new().add_modifier(modifiers).fg(fg),
                        ));
                    }
                    self.lines.push(line);
                }
                Err(_) => {
                    self.lines.push(Line::raw(code_line.to_owned()));
                }
            }
        }
    }
}

pub struct BreakParser<'a>(BlockParser<'a>);

impl<'a> BreakParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self(BlockParser::new(input))
    }
}

impl<'a> Iterator for BreakParser<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .borrow_mut()
            .filter(|(block, _)| matches!(block, BlockElement::Break))
            .map(|(_, range)| range.start)
            .next()
    }
}

fn calculate_scroll(scroll: usize, lines: usize, height: usize) -> usize {
    if lines <= height {
        0
    } else {
        usize::min(scroll, lines - height)
    }
}

#[derive(Debug)]
enum BlockElement<'a> {
    Paragraph { alignment: Alignment, text: &'a str },
    List { items: ListItems<'a> },
    Code { language: &'a str, text: &'a str },
    Break,
}

struct BlockParser<'a> {
    input: &'a str,
    chars: CustomCharIter<'a>,
}

impl<'a> BlockParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: CustomCharIter::new(input),
        }
    }

    fn parse_paragraph(
        &mut self,
        start: usize,
        alignment: Alignment,
    ) -> (BlockElement<'a>, Range<usize>) {
        let start_offset = match alignment {
            Alignment::Left => 0,
            Alignment::Center | Alignment::Right => 1,
        };
        let paragraph_start = start + start_offset;
        let (paragraph_end, end) = match self.chars.find_consecutive('\n', 2) {
            Some(i) => (i - 1, i + 1),
            None => (self.input.len(), self.input.len()),
        };

        return (
            BlockElement::Paragraph {
                alignment,
                text: self.input[paragraph_start..paragraph_end].trim(),
            },
            start..end,
        );
    }

    fn parse_list(&mut self, start: usize) -> (BlockElement<'a>, Range<usize>) {
        let list_start = start + 1;
        let (list_end, end) = match self.chars.find_consecutive('\n', 2) {
            Some(i) => (i - 1, i + 1),
            None => (self.input.len(), self.input.len()),
        };

        return (
            BlockElement::List {
                items: ListItems::new(self.input[list_start..list_end].trim()),
            },
            start..end,
        );
    }

    fn parse_code_block(&mut self, start: usize, ticks: usize) -> (BlockElement<'a>, Range<usize>) {
        let lang_start = start + ticks;
        let Some(i) = self.chars.find('\n') else {
            return (
                BlockElement::Code {
                    language: self.input[lang_start..].trim(),
                    text: "",
                },
                start..self.input.len(),
            );
        };

        let language = self.input[lang_start..i].trim();
        let code_start = i + 1;
        loop {
            let Some(code_end) = self.chars.find('\n') else {
                return (
                    BlockElement::Code {
                        language,
                        text: &self.input[code_start..],
                    },
                    start..self.input.len(),
                );
            };

            let end_ticks = self.chars.count_consecutive('`', usize::MAX);
            if end_ticks == ticks {
                let end = code_end + 1 + end_ticks;
                let mut chars = self.input[end..].chars();

                let Some(c) = chars.next() else {
                    return (
                        BlockElement::Code {
                            language,
                            text: &self.input[code_start..code_end],
                        },
                        start..end,
                    );
                };

                if c == '\n' {
                    let Some(c) = chars.next() else {
                        return (
                            BlockElement::Code {
                                language,
                                text: &self.input[code_start..code_end],
                            },
                            start..end + 1,
                        );
                    };

                    if c == '\n' {
                        self.chars.next();
                        self.chars.next();

                        return (
                            BlockElement::Code {
                                language,
                                text: &self.input[code_start..code_end],
                            },
                            start..end + 2,
                        );
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for BlockParser<'a> {
    type Item = (BlockElement<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.chars.next() {
            if c.is_whitespace() {
                continue;
            }

            if let Some('\n') | None = self.chars.previous() {
                let (block, range) = match c {
                    '|' => self.parse_paragraph(i, Alignment::Center),
                    '>' => self.parse_paragraph(i, Alignment::Right),
                    '`' => {
                        let ticks = 1 + self.chars.count_consecutive('`', usize::MAX);
                        if ticks >= 3 {
                            self.parse_code_block(i, ticks)
                        } else {
                            self.parse_paragraph(i, Alignment::Left)
                        }
                    }
                    '-' => {
                        let dashes = 1 + self.chars.count_consecutive('-', usize::MAX);
                        if dashes == 1 {
                            self.parse_list(i)
                        } else if dashes == 3 && self.chars.count_consecutive('\n', 2) == 2 {
                            (BlockElement::Break, i..i + dashes + 2)
                        } else {
                            self.parse_paragraph(i, Alignment::Left)
                        }
                    }
                    _ => self.parse_paragraph(i, Alignment::Left),
                };

                return Some((block, range));
            } else {
                return Some(self.parse_paragraph(i, Alignment::Left));
            }
        }

        None
    }
}

#[derive(Debug)]
struct ListItems<'a> {
    text: &'a str,
    chars: CustomCharIter<'a>,
    start: usize,
}

impl<'a> ListItems<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            chars: CustomCharIter::new(text),
            start: 0,
        }
    }
}

impl<'a> Iterator for ListItems<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.text.len() {
            return None;
        }

        let (end, next_start) = match self.chars.find_pattern('\n', '-') {
            Some(i) => (i - 1, i + 1),
            None => (self.text.len(), self.text.len()),
        };

        let item = self.text[self.start..end].trim();
        self.start = next_start;
        Some(item)
    }
}

#[derive(Debug, Clone, Copy)]
enum InlineTag {
    Normal,
    Bold,
    Italic,
}

struct InlineParser<'a> {
    input: &'a str,
    chars: CustomCharIter<'a>,
    start: usize,
    tag: InlineTag,
}

impl<'a> InlineParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: CustomCharIter::new(input),
            start: 0,
            tag: InlineTag::Normal,
        }
    }

    fn _continue_with(&mut self, input: &'a str) -> &mut Self {
        self.input = input;
        self.chars = CustomCharIter::new(input);
        self.start = 0;
        self
    }
}

impl<'a> Iterator for InlineParser<'a> {
    type Item = (InlineTag, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.input.len() {
            return None;
        }

        loop {
            match self.tag {
                InlineTag::Normal => loop {
                    let Some((i, c)) = self.chars.next() else {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((InlineTag::Normal, text));
                    };

                    match c {
                        '*' => {
                            if let Some(p) = self.chars.peek() {
                                if p != '*' && !p.is_whitespace() {
                                    self.tag = InlineTag::Bold;
                                    let text = &self.input[self.start..i];
                                    self.start = i + 1;
                                    if text.is_empty() {
                                        break;
                                    }
                                    return Some((InlineTag::Normal, text));
                                }
                            }
                        }
                        '_' => {
                            if let Some(p) = self.chars.peek() {
                                if p != '_' && !p.is_whitespace() {
                                    self.tag = InlineTag::Italic;
                                    let text = &self.input[self.start..i];
                                    self.start = i + 1;
                                    if text.is_empty() {
                                        break;
                                    }
                                    return Some((InlineTag::Normal, text));
                                }
                            }
                        }
                        _ => {}
                    }
                },
                InlineTag::Bold => {
                    let (text_end, next_start) = match self
                        .chars
                        .find_with_previous('*', |p| p != '*' && !p.is_whitespace())
                    {
                        Some(i) => {
                            self.tag = InlineTag::Normal;
                            (i, i + 1)
                        }
                        None => (self.input.len(), self.input.len()),
                    };
                    let text = &self.input[self.start..text_end];
                    self.start = next_start;
                    return Some((InlineTag::Bold, text));
                }
                InlineTag::Italic => {
                    let (text_end, next_start) = match self
                        .chars
                        .find_with_previous('_', |p| p != '_' && !p.is_whitespace())
                    {
                        Some(i) => {
                            self.tag = InlineTag::Normal;
                            (i, i + 1)
                        }
                        None => (self.input.len(), self.input.len()),
                    };
                    let text = &self.input[self.start..text_end];
                    self.start = next_start;
                    return Some((InlineTag::Italic, text));
                }
            }
        }
    }
}

#[derive(Debug)]
struct CustomCharIter<'a> {
    chars: Peekable<CharIndices<'a>>,
    current: Option<(usize, char)>,
    previous: Option<char>,
}

impl<'a> CustomCharIter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            chars: text.char_indices().peekable(),
            current: None,
            previous: None,
        }
    }

    fn _current(&self) -> Option<(usize, char)> {
        self.current
    }

    fn previous(&self) -> Option<char> {
        self.previous
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn next_if_eq(&mut self, c: char) -> Option<(usize, char)> {
        if let Some((_, peek)) = self.chars.peek() {
            if *peek == c {
                return self.next();
            }
        }
        None
    }

    fn find(&mut self, c: char) -> Option<usize> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if n == c {
                return Some(i);
            }
        }
    }

    fn find_with_previous(&mut self, c: char, func: impl Fn(char) -> bool) -> Option<usize> {
        loop {
            let Some(i) = self.find(c) else {
                return None;
            };

            if let Some(p) = self.previous {
                if func(p) {
                    return Some(i);
                }
            };
        }
    }

    fn find_pattern(&mut self, prev: char, next: char) -> Option<usize> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if let Some(p) = self.previous {
                if p == prev && n == next {
                    return Some(i);
                }
            }
        }
    }

    fn _find_pattern_by(&mut self, func: impl Fn(char, char) -> bool) -> Option<usize> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if let Some(p) = self.previous {
                if func(p, n) {
                    return Some(i);
                }
            }
        }
    }

    fn find_consecutive(&mut self, c: char, n: usize) -> Option<usize> {
        match n {
            0 => None,
            1 => self.find(c),
            _ => loop {
                if self.find(c).is_none() {
                    return None;
                };

                let mut count = 1;
                while let Some((i, _)) = self.next_if_eq(c) {
                    count += 1;
                    if count == n {
                        return Some(i);
                    }
                }
            },
        }
    }

    fn count_consecutive(&mut self, c: char, max: usize) -> usize {
        if max == 0 {
            return 0;
        }

        let mut count = 0;
        while self.next_if_eq(c).is_some() {
            count += 1;
            if count == max {
                break;
            }
        }
        count
    }
}

impl<'a> Iterator for CustomCharIter<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((i, n)) = self.chars.next() else {
            return None;
        };

        if let Some((_, c)) = self.current {
            self.previous = Some(c);
        }

        self.current = Some((i, n));
        self.current
    }
}

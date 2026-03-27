use std::{
    borrow::BorrowMut,
    hash::{DefaultHasher, Hash, Hasher},
    iter::Peekable,
    ops::Range,
    sync::LazyLock,
};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use super::Shortcut;

#[derive(Debug)]
pub struct Markup {
    width: usize,
    height: usize,
    hash: u64,
    scroll: usize,
    desired_scroll: Option<usize>,
    syntax_highlight_theme: &'static str,
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
    pub const SHORTCUTS: [Shortcut<'static>; 1] = [Shortcut::new("Scroll", "⮁")];

    pub const fn new(syntax_highlight_theme: &'static str) -> Self {
        Self {
            width: 0,
            height: 0,
            hash: 0,
            scroll: 0,
            desired_scroll: None,
            syntax_highlight_theme,
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

    pub fn render(&mut self, text: &str, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        self.height = area.height as usize;

        // todo: switch hasher?
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
                    BlockElement::Paragraph { text, alignment } => {
                        self.parse_text(text, alignment, "", "");
                    }
                    BlockElement::Code { language, text } => {
                        self.parse_code(language, text);
                    }
                    BlockElement::List { items } => {
                        for item in items {
                            self.parse_text(item, Alignment::Left, " • ", "   ");
                        }
                    }
                    BlockElement::Comment { .. } => continue,
                    BlockElement::Break => self.lines.push(
                        Line::styled("——————————", Style::new().fg(Color::DarkGray))
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
                line.render(line_area, buf);
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
                InlineTag::Normal => Style::new(),
                InlineTag::Bold => Style::new().bold(),
                InlineTag::Italic => Style::new().italic(),
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
                        if column + word_width > width {
                            if word_width > width / 2 {
                                // break word
                                for g_span in self.word_buffer.drain(..) {
                                    let g_width = g_span.width();
                                    if column + g_width > width {
                                        self.lines.push(line);
                                        (line, column) = new_line(wrap_indent, alignment);
                                    }
                                    line.push_span(g_span);
                                    column += g_width;
                                }
                            } else {
                                // push word to next line
                                self.lines.push(line);
                                (line, column) = new_line(wrap_indent, alignment);
                                line.extend(self.word_buffer.drain(..));
                                column += word_width;
                            }
                        } else {
                            line.extend(self.word_buffer.drain(..));
                            column += word_width;
                        }
                        line.push_span(Span::styled(" ", style));
                        column += 1;
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
            if column + word_width > width {
                if word_width > width / 2 {
                    // break word
                    for g_span in self.word_buffer.drain(..) {
                        let g_width = g_span.width();
                        if column + g_width > width {
                            self.lines.push(line);
                            (line, column) = new_line(wrap_indent, alignment);
                        }
                        line.push_span(g_span);
                        column += g_width;
                    }
                } else {
                    // push word to next line
                    self.lines.push(line);
                    (line, _) = new_line(wrap_indent, alignment);
                    line.extend(self.word_buffer.drain(..));
                }
            } else {
                line.extend(self.word_buffer.drain(..));
            }
        }

        self.lines.push(line);
    }

    fn parse_code(&mut self, language: &str, text: &str) {
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
            HighlightLines::new(syntax, &THEME_SET.themes[self.syntax_highlight_theme]);

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
    Paragraph { text: &'a str, alignment: Alignment },
    List { items: ListItems<'a> },
    Code { language: &'a str, text: &'a str },
    Comment { _text: &'a str },
    Break,
}

struct BlockParser<'a> {
    input: &'a str,
    graphemes: CustomGraphemeIter<'a>,
}

impl<'a> BlockParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            graphemes: CustomGraphemeIter::new(input),
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
        let (paragraph_end, end) = match self.graphemes.find_consecutive_by(|g| g.contains('\n'), 2)
        {
            Some((i, g)) => (i, i + g.len()),
            None => (self.input.len(), self.input.len()),
        };

        return (
            BlockElement::Paragraph {
                text: self.input[paragraph_start..paragraph_end].trim(),
                alignment,
            },
            start..end,
        );
    }

    fn parse_list(&mut self, start: usize) -> (BlockElement<'a>, Range<usize>) {
        let list_start = start + 1;
        let (list_end, end) = match self.graphemes.find_consecutive_by(|g| g.contains('\n'), 2) {
            Some((i, g)) => (i, i + g.len()),
            None => (self.input.len(), self.input.len()),
        };

        return (
            BlockElement::List {
                items: ListItems::new(self.input[list_start..list_end].trim()),
            },
            start..end,
        );
    }

    fn parse_comment(&mut self, start: usize) -> (BlockElement<'a>, Range<usize>) {
        let comment_start = start + 1;
        let (comment_end, end) = match self.graphemes.find_by(|g| g.contains('\n')) {
            Some((i, g)) => (i, i + g.len()),
            None => (self.input.len(), self.input.len()),
        };

        return (
            BlockElement::Comment {
                _text: self.input[comment_start..comment_end].trim(),
            },
            start..end,
        );
    }

    fn parse_code_block(&mut self, start: usize, ticks: usize) -> (BlockElement<'a>, Range<usize>) {
        let lang_start = start + ticks;
        let Some((i, g)) = self.graphemes.find_by(|g| g.contains('\n')) else {
            return (
                BlockElement::Code {
                    language: self.input[lang_start..].trim(),
                    text: "",
                },
                start..self.input.len(),
            );
        };

        let language = self.input[lang_start..i].trim();
        let code_start = i + g.len();
        loop {
            let Some((code_end, g)) = self.graphemes.find_by(|g| g.contains('\n')) else {
                return (
                    BlockElement::Code {
                        language,
                        text: &self.input[code_start..],
                    },
                    start..self.input.len(),
                );
            };

            let end_ticks = self.graphemes.count_consecutive("`", usize::MAX);
            if end_ticks == ticks {
                let end = code_end + g.len() + end_ticks;
                let mut graphemes = self.input[end..].grapheme_indices(true);

                let Some((i, g)) = graphemes.next() else {
                    return (
                        BlockElement::Code {
                            language,
                            text: &self.input[code_start..code_end],
                        },
                        start..end,
                    );
                };

                if g.contains('\n') {
                    let Some((i, g)) = graphemes.next() else {
                        return (
                            BlockElement::Code {
                                language,
                                text: &self.input[code_start..code_end],
                            },
                            start..i + g.len(),
                        );
                    };

                    if g.contains('\n') {
                        self.graphemes.next();
                        self.graphemes.next();

                        return (
                            BlockElement::Code {
                                language,
                                text: &self.input[code_start..code_end],
                            },
                            start..i + g.len(),
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
        while let Some((i, g)) = self.graphemes.next() {
            if g.chars().any(|c| c.is_whitespace()) {
                continue;
            }

            if let Some("\n") | Some("\r\n") | None = self.graphemes.previous() {
                let (block, range) = match g {
                    "|" => self.parse_paragraph(i, Alignment::Center),
                    ">" => self.parse_paragraph(i, Alignment::Right),
                    "#" => self.parse_comment(i),
                    "`" => {
                        let ticks = 1 + self.graphemes.count_consecutive("`", usize::MAX);
                        if ticks >= 3 {
                            self.parse_code_block(i, ticks)
                        } else {
                            self.parse_paragraph(i, Alignment::Left)
                        }
                    }
                    "-" => {
                        let dashes = 1 + self.graphemes.count_consecutive("-", usize::MAX);
                        if dashes == 1 {
                            self.parse_list(i)
                        } else if dashes == 3
                            && self.graphemes.count_consecutive_by(|g| g.contains('\n'), 2) == 2
                        {
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
    graphemes: CustomGraphemeIter<'a>,
    start: usize,
}

impl<'a> ListItems<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            graphemes: CustomGraphemeIter::new(text),
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

        let (end, next_start) = match self
            .graphemes
            .find_pattern_by(|prev, next| prev.contains('\n') && next == "-")
        {
            Some((i, p, n)) => (i - p.len(), i + n.len()),
            None => (self.text.len(), self.text.len()),
        };

        let item = self.text[self.start..end].trim();
        self.start = next_start;
        Some(item)
    }
}

// todo: rework with start/end tags?
#[derive(Debug, Clone, Copy)]
enum InlineTag {
    Normal,
    Bold,
    Italic,
}

struct InlineParser<'a> {
    input: &'a str,
    graphemes: CustomGraphemeIter<'a>,
    start: usize,
    tag: InlineTag,
}

impl<'a> InlineParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            graphemes: CustomGraphemeIter::new(input),
            start: 0,
            tag: InlineTag::Normal,
        }
    }

    fn _continue_with(&mut self, input: &'a str) -> &mut Self {
        self.input = input;
        self.graphemes = CustomGraphemeIter::new(input);
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
                    let Some((i, g)) = self.graphemes.next() else {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((InlineTag::Normal, text));
                    };

                    match g {
                        "*" => {
                            if let Some(p) = self.graphemes.peek() {
                                if p != "*" && !p.chars().any(|c| c.is_whitespace()) {
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
                        "_" => {
                            if let Some(p) = self.graphemes.peek() {
                                if p != "_" && !p.chars().any(|c| c.is_whitespace()) {
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
                    let (text_end, next_start) = match self.graphemes.find_with_previous("*", |p| {
                        p != "*" && !p.chars().any(|c| c.is_whitespace())
                    }) {
                        Some((i, g)) => {
                            self.tag = InlineTag::Normal;
                            (i, i + g.len())
                        }
                        None => (self.input.len(), self.input.len()),
                    };
                    let text = &self.input[self.start..text_end];
                    self.start = next_start;
                    return Some((InlineTag::Bold, text));
                }
                InlineTag::Italic => {
                    let (text_end, next_start) = match self.graphemes.find_with_previous("_", |p| {
                        p != "_" && !p.chars().any(|c| c.is_whitespace())
                    }) {
                        Some((i, g)) => {
                            self.tag = InlineTag::Normal;
                            (i, i + g.len())
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
struct CustomGraphemeIter<'a> {
    graphemes: Peekable<GraphemeIndices<'a>>,
    current: Option<(usize, &'a str)>,
    previous: Option<&'a str>,
}

impl<'a> CustomGraphemeIter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            graphemes: text.grapheme_indices(true).peekable(),
            current: None,
            previous: None,
        }
    }

    fn previous(&self) -> Option<&'a str> {
        self.previous
    }

    fn peek(&mut self) -> Option<&'a str> {
        self.graphemes.peek().map(|(_, g)| *g)
    }

    fn next_if(&mut self, func: impl Fn(&str) -> bool) -> Option<(usize, &'a str)> {
        if let Some((_, peek)) = self.graphemes.peek() {
            if func(peek) {
                return self.next();
            }
        }
        None
    }

    fn _next_if_eq(&mut self, g: &str) -> Option<(usize, &'a str)> {
        self.next_if(|n| n == g)
    }

    fn _find(&mut self, g: &str) -> Option<(usize, &'a str)> {
        self.find_by(|n| n == g)
    }

    fn find_by(&mut self, func: impl Fn(&str) -> bool) -> Option<(usize, &'a str)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if func(n) {
                return Some((i, n));
            }
        }
    }

    fn find_with_previous(
        &mut self,
        g: &str,
        prev_func: impl Fn(&str) -> bool,
    ) -> Option<(usize, &'a str)> {
        self.find_by_with_previous(|n| n == g, prev_func)
    }

    fn find_by_with_previous(
        &mut self,
        next_func: impl Fn(&str) -> bool,
        prev_func: impl Fn(&str) -> bool,
    ) -> Option<(usize, &'a str)> {
        loop {
            let Some((i, n)) = self.find_by(&next_func) else {
                return None;
            };

            if let Some(p) = self.previous {
                if prev_func(p) {
                    return Some((i, n));
                }
            };
        }
    }

    fn _find_pattern(&mut self, prev: &str, next: &str) -> Option<(usize, &'a str, &'a str)> {
        self.find_pattern_by(|p, n| p == prev && n == next)
    }

    fn find_pattern_by(
        &mut self,
        func: impl Fn(&str, &str) -> bool,
    ) -> Option<(usize, &'a str, &'a str)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if let Some(p) = self.previous {
                if func(p, n) {
                    return Some((i, p, n));
                }
            }
        }
    }

    fn _find_consecutive(&mut self, g: &str, n: usize) -> Option<(usize, &'a str)> {
        self.find_consecutive_by(|s| s == g, n)
    }

    fn find_consecutive_by(
        &mut self,
        func: impl Fn(&str) -> bool,
        n: usize,
    ) -> Option<(usize, &'a str)> {
        match n {
            0 => None,
            1 => self.find_by(func),
            _ => loop {
                if self.find_by(&func).is_none() {
                    return None;
                };

                let mut count = 1;
                while let Some((i, g)) = self.next_if(&func) {
                    count += 1;
                    if count == n {
                        return Some((i, g));
                    }
                }
            },
        }
    }

    fn count_consecutive(&mut self, g: &str, max: usize) -> usize {
        self.count_consecutive_by(|n| n == g, max)
    }

    fn count_consecutive_by(&mut self, g: impl Fn(&str) -> bool, max: usize) -> usize {
        if max == 0 {
            return 0;
        }

        let mut count = 0;
        while self.next_if(&g).is_some() {
            count += 1;
            if count == max {
                break;
            }
        }
        count
    }
}

impl<'a> Iterator for CustomGraphemeIter<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((i, n)) = self.graphemes.next() else {
            return None;
        };

        if let Some((_, c)) = self.current {
            self.previous = Some(c);
        }

        self.current = Some((i, n));
        self.current
    }
}

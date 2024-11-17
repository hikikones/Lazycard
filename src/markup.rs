use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter::Peekable,
    ops::Range,
    str::CharIndices,
    sync::LazyLock,
};

use ratatui::{prelude::*, widgets::WidgetRef};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use crate::utils::STYLE_LABEL;

#[derive(Debug)]
pub struct Markup {
    width: usize,
    height: usize,
    hash: u64,
    scroll: usize,
    desired_scroll: Option<usize>,
    lines: Vec<Line<'static>>,
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

    pub fn render_markup(&mut self, text: &str, area: Rect, buf: &mut Buffer) {
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
                        let mut line = Line::default().alignment(alignment);
                        let mut column = 0;

                        for (tag, span) in InlineParser::new(text) {
                            let style = match tag {
                                InlineTag::Text => Style::new(),
                                InlineTag::Bold => Style::new().bold(),
                                InlineTag::Italic => Style::new().italic(),
                            };

                            for word in span.split_whitespace() {
                                let word_width = unicode_width::UnicodeWidthStr::width(word);
                                // todo: word_width > width
                                if column + word_width > width {
                                    self.lines.push(line);
                                    line = Line::default().alignment(alignment);
                                    column = 0;
                                }

                                line.push_span(Span::styled(word.to_owned(), style));
                                line.push_span(Span::styled(" ", style));
                                column += word_width + 1;
                            }

                            line.spans.pop();
                            line.push_span(Span::raw(" "));
                        }

                        self.lines.push(line);
                    }
                    BlockElement::Code { language, text } => {
                        static SYNTAX_SET: LazyLock<SyntaxSet> =
                            LazyLock::new(|| SyntaxSet::load_defaults_newlines());
                        static THEME_SET: LazyLock<ThemeSet> =
                            LazyLock::new(|| ThemeSet::load_defaults());

                        let syntax = if language.is_empty() {
                            SYNTAX_SET.find_syntax_plain_text()
                        } else {
                            SYNTAX_SET
                                .find_syntax_by_token(language)
                                .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
                        };
                        let mut highlighter =
                            HighlightLines::new(syntax, &THEME_SET.themes["base16-eighties.dark"]);

                        for code_line in LinesWithEndings::from(text.replace('\t', "    ").as_str())
                        {
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
                                        let fg = Color::Rgb(
                                            style.foreground.r,
                                            style.foreground.g,
                                            style.foreground.b,
                                        );
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
                    BlockElement::Break => self
                        .lines
                        .push(Line::styled("——————————", STYLE_LABEL).alignment(Alignment::Center)),
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
}

fn calculate_scroll(scroll: usize, lines: usize, height: usize) -> usize {
    if lines <= height {
        0
    } else {
        usize::min(scroll, lines - height)
    }
}

#[derive(Debug)]
pub enum BlockElement<'a> {
    Paragraph { alignment: Alignment, text: &'a str },
    Code { language: &'a str, text: &'a str },
    Break,
}

pub struct BlockParser<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    prev: Option<char>,
}

impl<'a> BlockParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            prev: None,
        }
    }

    fn parse_paragraph(
        &mut self,
        start: usize,
        alignment: Alignment,
    ) -> (BlockElement<'a>, Range<usize>) {
        let offset = match alignment {
            Alignment::Left => 0,
            Alignment::Center | Alignment::Right => 1,
        };
        let paragraph_start = start + offset;

        loop {
            if self.chars.find(|&(_, c)| c == '\n').is_none() {
                return (
                    BlockElement::Paragraph {
                        alignment,
                        text: self.input[paragraph_start..].trim(),
                    },
                    start..self.input.len(),
                );
            };

            if let Some((end, '\n')) = self.chars.next() {
                return (
                    BlockElement::Paragraph {
                        alignment,
                        text: self.input[paragraph_start..end - 1].trim(),
                    },
                    start..end + 1,
                );
            }
        }
    }

    fn parse_code_block(&mut self, start: usize, ticks: usize) -> (BlockElement<'a>, Range<usize>) {
        let lang_start = start + ticks;
        let Some((i, _)) = self.chars.find(|(_, c)| *c == '\n') else {
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
            let Some((code_end, _)) = self.chars.find(|(_, c)| *c == '\n') else {
                return (
                    BlockElement::Code {
                        language,
                        text: &self.input[code_start..],
                    },
                    start..self.input.len(),
                );
            };

            let end_ticks = self.count_consecutive('`');
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

    fn count_consecutive(&mut self, c: char) -> usize {
        let mut count = 0;
        while self.chars.next_if(|(_, cc)| *cc == c).is_some() {
            count += 1;
        }
        count
    }
}

impl<'a> Iterator for BlockParser<'a> {
    type Item = (BlockElement<'a>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.chars.next() {
            if c.is_whitespace() {
                self.prev = Some(c);
                continue;
            }

            let (block, range) = match c {
                '|' => {
                    if let Some('\n') | None = self.prev {
                        self.parse_paragraph(i, Alignment::Center)
                    } else {
                        self.parse_paragraph(i, Alignment::Left)
                    }
                }
                '>' => {
                    if let Some('\n') | None = self.prev {
                        self.parse_paragraph(i, Alignment::Right)
                    } else {
                        self.parse_paragraph(i, Alignment::Left)
                    }
                }
                '`' => {
                    if let Some('\n') | None = self.prev {
                        let ticks = 1 + self.count_consecutive('`');
                        if ticks >= 3 {
                            self.parse_code_block(i, ticks)
                        } else {
                            self.parse_paragraph(i, Alignment::Left)
                        }
                    } else {
                        self.parse_paragraph(i, Alignment::Left)
                    }
                }
                '-' => {
                    if let Some('\n') | None = self.prev {
                        let dashes = 1 + self.count_consecutive('-');
                        if dashes == 1 {
                            // todo: list item
                            self.parse_paragraph(i, Alignment::Left)
                        } else if dashes == 3 && self.count_consecutive('\n') >= 2 {
                            (BlockElement::Break, i..i + dashes + 2)
                        } else {
                            self.parse_paragraph(i, Alignment::Left)
                        }
                    } else {
                        self.parse_paragraph(i, Alignment::Left)
                    }
                }
                _ => self.parse_paragraph(i, Alignment::Left),
            };

            self.prev = None;
            return Some((block, range));
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
enum InlineTag {
    Text,
    Bold,
    Italic,
}

struct InlineParser<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    start: usize,
    tag: InlineTag,
}

impl<'a> InlineParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            start: 0,
            tag: InlineTag::Text,
        }
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
                InlineTag::Text => loop {
                    let Some((_, c)) = self.chars.next() else {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((InlineTag::Text, text));
                    };

                    match c {
                        '*' => {
                            if let Some((i, _)) = self.chars.next_if(|&(_, c)| c == '*') {
                                self.tag = InlineTag::Bold;
                                let text = &self.input[self.start..i - 1];
                                self.start = i + 1;
                                if text.is_empty() {
                                    break;
                                }
                                return Some((InlineTag::Text, text));
                            }
                        }
                        '_' => {
                            if let Some((i, _)) = self.chars.next_if(|&(_, c)| c == '_') {
                                self.tag = InlineTag::Italic;
                                let text = &self.input[self.start..i - 1];
                                self.start = i + 1;
                                if text.is_empty() {
                                    break;
                                }
                                return Some((InlineTag::Text, text));
                            }
                        }
                        _ => {}
                    }
                },
                InlineTag::Bold => loop {
                    if self.chars.find(|&(_, c)| c == '*').is_none() {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((InlineTag::Bold, text));
                    };

                    if let Some((i, '*')) = self.chars.next() {
                        self.tag = InlineTag::Text;
                        let text = &self.input[self.start..i - 1];
                        self.start = i + 1;
                        return Some((InlineTag::Bold, text));
                    }
                },
                InlineTag::Italic => loop {
                    if self.chars.find(|&(_, c)| c == '_').is_none() {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((InlineTag::Bold, text));
                    };

                    if let Some((i, '_')) = self.chars.next() {
                        self.tag = InlineTag::Text;
                        let text = &self.input[self.start..i - 1];
                        self.start = i + 1;
                        return Some((InlineTag::Italic, text));
                    }
                },
            }
        }
    }
}

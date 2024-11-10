use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter::Peekable,
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

#[derive(Debug)]
pub struct Markup {
    lines: Vec<Line<'static>>,
    width: usize,
    height: usize,
    scroll: usize,
    hash: u64,
}

pub enum ScrollMove {
    Up,
    Down,
    Start,
    _End,
}

impl Markup {
    pub const fn new() -> Self {
        Self {
            lines: Vec::new(),
            width: 0,
            height: 0,
            scroll: 0,
            hash: 0,
        }
    }

    pub fn scroll(&mut self, sm: ScrollMove) -> bool {
        let lines = self.lines.len();
        let height = self.height;

        match sm {
            ScrollMove::Up => self.set_scroll(self.scroll.saturating_sub(1), lines, height),
            ScrollMove::Down => self.set_scroll(self.scroll.saturating_add(1), lines, height),
            ScrollMove::Start => self.set_scroll(0, lines, height),
            ScrollMove::_End => self.set_scroll(usize::MAX, lines, height),
        }
    }

    fn set_scroll(&mut self, n: usize, lines: usize, height: usize) -> bool {
        let current_scroll = self.scroll;

        let new_scroll = if lines <= height {
            0
        } else {
            usize::min(n, lines - height)
        };
        self.scroll = new_scroll;

        current_scroll != new_scroll
    }

    pub fn render_markup(&mut self, text: &str, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let height = area.height as usize;

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        self.height = height;

        if self.width != width || self.hash != hash {
            self.lines.clear();
            self.width = width;
            self.hash = hash;

            // Process markup
            for block in BlockParser::new(text) {
                match block {
                    BlockElement::Paragraph { alignment, text } => {
                        let ansi_markup = InlineParser::new(text).into_ansi().replace('\n', " ");
                        let mut ansi_parser = AnsiParser::new("");
                        let mut wrap_opts = textwrap::Options::new(width);
                        wrap_opts.word_splitter = textwrap::WordSplitter::NoHyphenation;
                        wrap_opts.wrap_algorithm = textwrap::WrapAlgorithm::FirstFit;

                        for wrapped_line in textwrap::fill(&ansi_markup, wrap_opts).lines() {
                            let mut line = Line::default().alignment(alignment);
                            ansi_parser.continue_with(wrapped_line);
                            for (tag, span) in &mut ansi_parser {
                                let style = match tag {
                                    AnsiTag::Text => Style::new(),
                                    AnsiTag::Bold => Style::new().bold(),
                                    AnsiTag::Italic => Style::new().italic(),
                                };
                                line.push_span(Span::styled(span.to_owned(), style));
                            }
                            self.lines.push(line);
                        }
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

                        for code_line in LinesWithEndings::from(text) {
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
                }
                self.lines.push(Line::default());
            }
            self.lines.pop();

            // Update scroll
            self.set_scroll(self.scroll, self.lines.len(), height);
        }

        // Render lines
        let mut line_area = Rect { height: 1, ..area };
        self.lines
            .iter()
            .skip(self.scroll)
            .take(height)
            .for_each(|line| {
                line.render_ref(line_area, buf);
                line_area.y += 1;
            });
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll = 0;
        self.width = 0;
        self.height = 0;
        self.hash = 0;
    }
}

#[derive(Debug)]
enum BlockElement<'a> {
    Paragraph { alignment: Alignment, text: &'a str },
    Code { language: &'a str, text: &'a str },
}

struct BlockParser<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> BlockParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
        }
    }
}

impl<'a> Iterator for BlockParser<'a> {
    type Item = BlockElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.chars.next() {
            match c {
                '\n' | ' ' | '\t' => {
                    continue;
                }
                '`' => {
                    let ticks = 1 + count_ticks(&mut self.chars);
                    if ticks < 3 {
                        return Some(parse_paragraph(i, c, self.input, &mut self.chars));
                    }
                    return Some(parse_code_block(i, ticks, self.input, &mut self.chars));
                }
                _ => {
                    return Some(parse_paragraph(i, c, self.input, &mut self.chars));
                }
            }
        }

        None
    }
}

fn parse_paragraph<'a>(
    i: usize,
    c: char,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> BlockElement<'a> {
    let (alignment, offset) = {
        if c == '>' {
            (Alignment::Right, 1)
        } else if c == '|' {
            (Alignment::Center, 1)
        } else {
            (Alignment::Left, 0)
        }
    };

    let paragraph_start = i + offset;
    loop {
        if chars.find(|&(_, c)| c == '\n').is_none() {
            return BlockElement::Paragraph {
                alignment,
                text: input[paragraph_start..].trim(),
            };
        };

        if let Some((ni, '\n')) = chars.next() {
            return BlockElement::Paragraph {
                alignment,
                text: input[paragraph_start..ni - 1].trim(),
            };
        }
    }
}

fn parse_code_block<'a>(
    i: usize,
    ticks: usize,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> BlockElement<'a> {
    let lang_start = i + ticks;
    let Some((i, _)) = chars.find(|(_, c)| *c == '\n') else {
        return BlockElement::Code {
            language: input[lang_start..].trim(),
            text: "",
        };
    };

    let language = input[lang_start..i].trim();
    let code_start = i + 1;
    loop {
        let Some((end, _)) = chars.find(|(_, c)| *c == '\n') else {
            return BlockElement::Code {
                language,
                text: &input[code_start..],
            };
        };

        let end_ticks = count_ticks(chars);
        if end_ticks == ticks {
            let Some((_, c)) = chars.next() else {
                return BlockElement::Code {
                    language,
                    text: &input[code_start..end],
                };
            };

            if c == '\n' {
                return BlockElement::Code {
                    language,
                    text: &input[code_start..end],
                };
            }
        }
    }
}

fn count_ticks(chars: &mut Peekable<CharIndices>) -> usize {
    let mut count = 0;
    while chars.next_if(|(_, c)| *c == '`').is_some() {
        count += 1;
    }
    count
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

    fn _continue_with(&mut self, input: &'a str) {
        self.input = input;
        self.chars = input.char_indices().peekable();
        self.start = 0;
    }

    fn into_ansi(self) -> String {
        const ANSI_BOLD: &str = "\u{1b}[1m";
        const ANSI_ITALIC: &str = "\u{1b}[3m";
        const ANSI_RESET: &str = "\u{1b}[0m";

        let mut ansi = String::with_capacity(self.input.len());
        for (tag, span) in self {
            match tag {
                InlineTag::Text => {
                    ansi.push_str(span);
                }
                InlineTag::Bold => {
                    ansi.extend([ANSI_BOLD, span, ANSI_RESET]);
                }
                InlineTag::Italic => {
                    ansi.extend([ANSI_ITALIC, span, ANSI_RESET]);
                }
            }
        }
        ansi
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

#[derive(Debug, Clone, Copy)]
enum AnsiTag {
    Text,
    Bold,
    Italic,
}

struct AnsiParser<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    start: usize,
    tag: AnsiTag,
}

impl<'a> AnsiParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            start: 0,
            tag: AnsiTag::Text,
        }
    }

    fn continue_with(&mut self, input: &'a str) {
        self.input = input;
        self.chars = input.char_indices().peekable();
        self.start = 0;
    }
}

impl<'a> Iterator for AnsiParser<'a> {
    type Item = (AnsiTag, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.input.len() {
            return None;
        }

        loop {
            match self.tag {
                AnsiTag::Text => {
                    let Some((end, _)) = self.chars.find(|(_, c)| *c == '\x1b') else {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((self.tag, text));
                    };

                    if self.chars.next_if(|(_, c)| *c == '[').is_none() {
                        continue;
                    }

                    let Some(tag) =
                        self.chars
                            .next_if(|&(_, c)| c == '1' || c == '3')
                            .map(|(_, c)| {
                                if c == '1' {
                                    AnsiTag::Bold
                                } else {
                                    AnsiTag::Italic
                                }
                            })
                    else {
                        continue;
                    };

                    let Some((i, _)) = self.chars.next_if(|(_, c)| *c == 'm') else {
                        continue;
                    };

                    let text = &self.input[self.start..end];
                    self.start = i + 1;
                    self.tag = tag;

                    if !text.is_empty() {
                        return Some((AnsiTag::Text, text));
                    }
                }
                AnsiTag::Bold | AnsiTag::Italic => {
                    let Some((end, _)) = self.chars.find(|(_, c)| *c == '\x1b') else {
                        let text = &self.input[self.start..];
                        self.start = self.input.len();
                        return Some((self.tag, text));
                    };

                    if self.chars.next_if(|(_, c)| *c == '[').is_none() {
                        continue;
                    }
                    if self.chars.next_if(|(_, c)| *c == '0').is_none() {
                        continue;
                    }
                    let Some((i, _)) = self.chars.next_if(|(_, c)| *c == 'm') else {
                        continue;
                    };

                    let tag = self.tag;
                    let text = &self.input[self.start..end];
                    self.start = i + 1;
                    self.tag = AnsiTag::Text;
                    return Some((tag, text));
                }
            }
        }
    }
}

use std::{
    borrow::BorrowMut,
    hash::{Hash, Hasher},
    iter::Peekable,
    ops::Range,
    str::CharIndices,
};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::{
    ansi::{AnsiEvent, AnsiParser, AnsiTag, AnsiWriter},
    kitty_graphics::{Dimensions, KittyGraphics, ResizeMode},
    text_segment::TextSegment,
    utils,
};

// todo: desired scroll

pub struct Markup {
    items: Vec<Item>,
    ansi: AnsiWriter,
    buffer: String,
    code_highlighter: CodeHighlighter,
    text_segment: TextSegment,
    scroll: u16,
    total_lines: u16,
    area: Rect,
    hash: u64,
    id_start: u32,
    id_counter: u32,
}

#[derive(Debug, Clone)]
enum Item {
    Paragraph {
        text: Range<usize>,
        alignment: Alignment,
    },
    ListItem {
        text: Range<usize>,
    },
    Code {
        text: Range<usize>,
        _language: Range<usize>,
    },
    Image {
        id: u32,
        dims: Dimensions,
    },
    Break,
    EmptyLine,
}

const LIST_ITEM_INDENT: &str = "  • ";
const LIST_ITEM_INDENT_WIDTH: u16 = 4;

struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    dark: bool,
}

impl CodeHighlighter {
    fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            dark: true,
        }
    }

    fn highlight(&self, language: &str, code: &str, mut f: impl FnMut(&str, Option<(u8, u8, u8)>)) {
        let syntax = if language.is_empty() {
            self.syntax_set.find_syntax_plain_text()
        } else {
            self.syntax_set
                .find_syntax_by_token(language)
                .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
        };
        let theme_name = if self.dark {
            "base16-eighties.dark"
        } else {
            "InspiredGitHub"
        };

        let mut highlighter = HighlightLines::new(syntax, &self.theme_set.themes[theme_name]);
        for code_line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(code_line, &self.syntax_set) {
                Ok(spans) => {
                    for (style, span) in spans {
                        let syntect::highlighting::Color { r, g, b, .. } = style.foreground;
                        f(span, Some((r, g, b)));
                    }
                }
                Err(_) => {
                    f(code_line, None);
                }
            }
        }
    }
}

impl Markup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            ansi: AnsiWriter::new(),
            buffer: String::new(),
            code_highlighter: CodeHighlighter::new(),
            text_segment: TextSegment::new(),
            scroll: 0,
            total_lines: 0,
            area: Rect::ZERO,
            hash: 0,
            id_start: 90,
            id_counter: 0,
        }
    }

    pub fn input(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> bool {
        match key {
            KeyCode::Down => {
                self.scroll += 1;
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.scroll += self.area.height;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(self.area.height);
            }
            KeyCode::Home => {
                self.scroll = 0;
            }
            KeyCode::End => {
                self.scroll = u16::MAX;
            }
            _ => {}
        }

        // todo: return only true when scroll differs
        true
    }

    pub fn render(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        text: &str,
        kitty: &mut KittyGraphics,
    ) {
        // Delete existing images
        self.delete_images(kitty).unwrap();

        let hash = {
            let mut hasher = ahash::AHasher::default();
            text.hash(&mut hasher);
            hasher.finish()
        };

        if self.area != area || self.hash != hash {
            self.area = area;
            self.hash = hash;
            self.compute(text, area.width, kitty);
        }

        fn is_in_viewport(curr_line: u16, top: u16, bot: u16) -> bool {
            curr_line >= top && curr_line < bot
        }

        fn render_ansi_line(
            area: Rect,
            buf: &mut Buffer,
            s: &str,
            text_segment: &mut TextSegment,
            style: &mut Style,
            alignment: Alignment,
        ) {
            for event in AnsiParser::new(s) {
                match event {
                    AnsiEvent::Text(s) => {
                        text_segment.push_str(s, *style);
                    }
                    AnsiEvent::Tag(tag) => match tag {
                        AnsiTag::Reset => {
                            *style = Style::new();
                        }
                        AnsiTag::Bold => {
                            style.add_modifier.insert(Modifier::BOLD);
                        }
                        AnsiTag::Italic => {
                            style.add_modifier.insert(Modifier::ITALIC);
                        }
                        AnsiTag::NotBold => {
                            style.add_modifier.remove(Modifier::BOLD);
                        }
                        AnsiTag::NotItalic => {
                            style.add_modifier.remove(Modifier::ITALIC);
                        }
                        AnsiTag::FgTrueColor(r, g, b) => {
                            style.fg = Some(Color::Rgb(r, g, b));
                        }
                        _ => {}
                    },
                }
            }
            text_segment.set_alignment(alignment).render(area, buf);
            text_segment.clear();
        }

        // Restrict scroll
        self.scroll = self
            .scroll
            .min(self.total_lines.saturating_sub(area.height));

        // Setup
        let top_y = area.y;
        let viewport_top = self.scroll;
        let viewport_bot = self.scroll + area.height;
        let mut lines_counter = 0;

        // Render
        for item in self.items.iter().cloned() {
            match item {
                Item::Paragraph { text, alignment } => {
                    let text = &self.buffer[text];
                    let mut style = Style::new();
                    for line in text.lines() {
                        if is_in_viewport(lines_counter, viewport_top, viewport_bot) {
                            render_ansi_line(
                                area,
                                buf,
                                line,
                                &mut self.text_segment,
                                &mut style,
                                alignment,
                            );
                            area.y += 1;
                            area.height = area.height.saturating_sub(1);
                        }

                        lines_counter += 1;
                    }
                }
                Item::ListItem { text } => {
                    let text = &self.buffer[text];
                    let mut style = Style::new();

                    for (i, line) in text.lines().enumerate() {
                        if is_in_viewport(lines_counter, viewport_top, viewport_bot) {
                            if i == 0 {
                                buf.set_stringn(
                                    area.x,
                                    area.y,
                                    LIST_ITEM_INDENT,
                                    LIST_ITEM_INDENT_WIDTH as usize,
                                    style,
                                );
                            }
                            render_ansi_line(
                                Rect {
                                    x: area.x + LIST_ITEM_INDENT_WIDTH,
                                    width: area.width.saturating_sub(LIST_ITEM_INDENT_WIDTH),
                                    ..area
                                },
                                buf,
                                line,
                                &mut self.text_segment,
                                &mut style,
                                Alignment::Left,
                            );
                            area.y += 1;
                            area.height = area.height.saturating_sub(1);
                        }

                        lines_counter += 1;
                    }
                }
                Item::Code { text, .. } => {
                    let text = &self.buffer[text];
                    let mut style = Style::new();

                    for line in text.lines() {
                        if is_in_viewport(lines_counter, viewport_top, viewport_bot) {
                            render_ansi_line(
                                area,
                                buf,
                                line,
                                &mut self.text_segment,
                                &mut style,
                                Alignment::Left,
                            );
                            area.y += 1;
                            area.height = area.height.saturating_sub(1);
                        }

                        lines_counter += 1;
                    }
                }
                Item::Image { id, dims } => {
                    let max_width = kitty.width(area.width);
                    let resized_dims = KittyGraphics::resize(dims, dims.width(max_width));
                    let resized_area = kitty.area(resized_dims);

                    if is_in_viewport(
                        lines_counter,
                        viewport_top.saturating_sub(resized_area.rows),
                        viewport_bot,
                    ) {
                        let mut available_rows = {
                            let curr_height = area.height;
                            let post_height = area.height.saturating_sub(resized_area.rows);
                            curr_height - post_height
                        };

                        let is_top = area.y == top_y;
                        if is_top {
                            let outside = lines_counter.abs_diff(self.scroll);
                            available_rows = available_rows.saturating_sub(outside);
                        }

                        let image_area = Rect {
                            height: available_rows,
                            ..area
                        };
                        kitty.render(
                            image_area,
                            buf,
                            id,
                            dims,
                            ResizeMode::FitWidthCropHeight(is_top),
                            utils::Alignment::CenterHorizontal,
                        );
                        // ratatui::widgets::Block::bordered().render(image_area, buf);

                        area.y += available_rows;
                        area.height = area.height.saturating_sub(available_rows);
                    }

                    lines_counter += resized_area.rows;
                }
                Item::Break => {
                    if is_in_viewport(lines_counter, viewport_top, viewport_bot) {
                        let half = area.width / 2;
                        let mut x = area.x + half / 2;
                        for _ in 0..half {
                            (x, _) = buf.set_stringn(x, area.y, "—", 1, Style::new());
                        }
                        area.y += 1;
                        area.height = area.height.saturating_sub(1);
                    }

                    lines_counter += 1;
                }
                Item::EmptyLine => {
                    if is_in_viewport(lines_counter, viewport_top, viewport_bot) {
                        area.y += 1;
                        area.height = area.height.saturating_sub(1);
                    }

                    lines_counter += 1;
                }
            }
        }

        // Store total lines count
        self.total_lines = lines_counter;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.ansi.clear();
        self.buffer.clear();
        self.scroll = 0;
        // TODO: self.desired_scroll = None;
        self.area = Rect::ZERO;
        self.hash = 0;
    }

    pub fn delete_images(&self, kitty: &KittyGraphics) -> std::io::Result<()> {
        // let range = 1..(self.id_counter + 1);
        // kitty.delete_ids(range)
        if self.id_counter > 0 {
            return kitty.delete_range(
                self.id_start,
                self.id_start + self.id_counter.saturating_sub(1),
            );
        }
        Ok(())
    }

    fn compute(&mut self, text: &str, width: u16, kitty: &mut KittyGraphics) {
        self.items.clear();
        self.buffer.clear();
        self.id_counter = 0;

        for (block, _) in BlockParser::new(text) {
            match block {
                BlockElement::Paragraph { text, alignment } => {
                    let range = self.parse_text(text, width);
                    self.items.push(Item::Paragraph {
                        text: range,
                        alignment,
                    });
                }
                BlockElement::List { items } => {
                    for item in items {
                        let range =
                            self.parse_text(item, width.saturating_sub(LIST_ITEM_INDENT_WIDTH));
                        self.items.push(Item::ListItem { text: range });
                    }
                }
                BlockElement::Code { language, text } => {
                    self.code_highlighter
                        .highlight(language, text, |span, color| match color {
                            Some((r, g, b)) => {
                                self.ansi.push_tag(AnsiTag::FgTrueColor(r, g, b));
                                self.ansi.push_str(span);
                                self.ansi.push_tag(AnsiTag::Reset);
                            }
                            None => {
                                self.ansi.push_str(span);
                            }
                        });

                    // Store results in text buffer
                    let start = self.buffer.len();
                    self.buffer.push_str(language);
                    let middle = self.buffer.len();
                    self.buffer.push_str(self.ansi.as_str());
                    let end = self.buffer.len();
                    self.ansi.clear();
                    self.items.push(Item::Code {
                        _language: start..middle,
                        text: middle..end,
                    });
                }
                BlockElement::Image { description, path } => {
                    kitty.load(path).unwrap();
                    let id = self.id_start + self.id_counter;
                    let dims = kitty.encode(id).unwrap();
                    self.items.push(Item::Image { id, dims });
                    self.id_counter += 1;

                    if !description.is_empty() {
                        let range = self.parse_text(description, width);
                        self.items.push(Item::Paragraph {
                            text: range,
                            alignment: Alignment::Center,
                        });
                    }
                }
                BlockElement::Break => {
                    self.items.push(Item::Break);
                }
                BlockElement::Comment { .. } => continue,
            }

            // Add empty line between each block element
            self.items.push(Item::EmptyLine);
        }

        // Remove last empty line
        self.items.pop();
    }

    fn parse_text(&mut self, text: &str, width: u16) -> Range<usize> {
        for event in InlineParser::new(text) {
            match event {
                InlineEvent::Text(s) => self.ansi.push_str(s),
                InlineEvent::Tag(tag) => match tag {
                    InlineTag::BoldStart => self.ansi.push_tag(AnsiTag::Bold),
                    InlineTag::BoldEnd => self.ansi.push_tag(AnsiTag::NotBold),
                    InlineTag::ItalicStart => self.ansi.push_tag(AnsiTag::Italic),
                    InlineTag::ItalicEnd => self.ansi.push_tag(AnsiTag::NotItalic),
                },
            }
        }

        // Break text into lines using textwrap which ignores ansi codes
        textwrap::fill_inplace(self.ansi.inner_mut(), width as usize);

        // Push text to buffer and use later with returned range
        let start = self.buffer.len();
        self.buffer.push_str(self.ansi.as_str());
        let end = self.buffer.len();
        self.ansi.clear();
        start..end
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

#[derive(Debug)]
enum BlockElement<'a> {
    Paragraph { text: &'a str, alignment: Alignment },
    List { items: ListItems<'a> },
    Code { language: &'a str, text: &'a str },
    Image { description: &'a str, path: &'a str },
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

    fn parse_image(&mut self, start: usize) -> (BlockElement<'a>, Range<usize>) {
        let Some((start_descr, "[")) = self.graphemes.next() else {
            return self.parse_paragraph(start, Alignment::Left);
        };
        let Some((end_descr, _)) = self.graphemes.find("]") else {
            return self.parse_paragraph(start, Alignment::Left);
        };

        let Some((start_path, "(")) = self.graphemes.next() else {
            return self.parse_paragraph(start, Alignment::Left);
        };
        let Some((end_path, _)) = self.graphemes.find(")") else {
            return self.parse_paragraph(start, Alignment::Left);
        };

        let description = self.input[start_descr + 1..end_descr].trim();
        let path = self.input[start_path + 1..end_path].trim();

        let Some((end, g)) = self.graphemes.next() else {
            return (
                BlockElement::Image { description, path },
                start..self.input.len(),
            );
        };

        if !g.contains("\n") {
            return self.parse_paragraph(start, Alignment::Left);
        }

        let Some((_, g)) = self.graphemes.next() else {
            return (BlockElement::Image { description, path }, start..end);
        };

        if !g.contains("\n") {
            return self.parse_paragraph(start, Alignment::Left);
        }

        return (BlockElement::Image { description, path }, start..end);
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
                    "!" => self.parse_image(i),
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

    fn next_if_eq(&mut self, g: &str) -> Option<(usize, &'a str)> {
        self.next_if(|n| n == g)
    }

    fn find(&mut self, g: &str) -> Option<(usize, &'a str)> {
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

    fn find_pattern(&mut self, prev: &str, next: &str) -> Option<(usize, &'a str, &'a str)> {
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

    fn find_consecutive(&mut self, g: &str, n: usize) -> Option<(usize, &'a str)> {
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

#[derive(Debug, Clone, Copy)]
enum InlineEvent<'a> {
    Text(&'a str),
    Tag(InlineTag),
}

#[derive(Debug, Clone, Copy)]
enum InlineTag {
    BoldStart,
    BoldEnd,
    ItalicStart,
    ItalicEnd,
}

struct InlineParser<'a> {
    input: &'a str,
    chars: CustomCharsIter<'a>,
    start: usize,
    tag: Option<InlineTag>,
    bold: bool,
    italic: bool,
}

impl<'a> InlineParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: CustomCharsIter::new(input),
            start: 0,
            tag: None,
            bold: false,
            italic: false,
        }
    }
}

impl<'a> Iterator for InlineParser<'a> {
    type Item = InlineEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tag) = self.tag.take() {
            return Some(InlineEvent::Tag(tag));
        }

        /// Looks for valid inline tag from the current, previous and next chars.
        /// If valid, either a start tag or end tag is returned based on the tag condition.
        /// That is, if tag condition is true, then the end tag will be returned,
        /// since this implies that a start tag has been found earlier.
        fn parse_tag(
            curr: char,
            prev: Option<char>,
            next: Option<char>,
            tag_start: InlineTag,
            tag_end: InlineTag,
            tag_cond: &mut bool,
        ) -> Option<InlineTag> {
            match (prev, next) {
                // Look for both tags based on tag condition
                (Some(prev), Some(next)) => {
                    if *tag_cond {
                        if !prev.is_whitespace() && prev != curr {
                            *tag_cond = false;
                            return Some(tag_end);
                        }
                    } else {
                        if !next.is_whitespace() && next != curr {
                            *tag_cond = true;
                            return Some(tag_start);
                        }
                    }
                }
                // First char, only need to look for start tag
                (None, Some(next)) => {
                    if !*tag_cond {
                        if !next.is_whitespace() && next != curr {
                            *tag_cond = true;
                            return Some(tag_start);
                        }
                    }
                }
                // Last char, only need to look for end tag
                (Some(prev), None) => {
                    if *tag_cond {
                        if !prev.is_whitespace() && prev != curr {
                            *tag_cond = false;
                            return Some(tag_end);
                        }
                    }
                }
                // Nothing to look for
                (None, None) => {}
            }

            None
        }

        while let Some((i, c)) = self.chars.next() {
            let tag = match c {
                '*' => parse_tag(
                    c,
                    self.chars.previous,
                    self.chars.peek(),
                    InlineTag::BoldStart,
                    InlineTag::BoldEnd,
                    &mut self.bold,
                ),
                '_' => parse_tag(
                    c,
                    self.chars.previous,
                    self.chars.peek(),
                    InlineTag::ItalicStart,
                    InlineTag::ItalicEnd,
                    &mut self.italic,
                ),
                _ => None,
            };

            if let Some(tag) = tag {
                let text = &self.input[self.start..i];
                self.start = i + c.len_utf8();
                if text.is_empty() {
                    return Some(InlineEvent::Tag(tag));
                } else {
                    self.tag = Some(tag);
                    return Some(InlineEvent::Text(text));
                }
            }
        }

        let remaining = &self.input[self.start..];
        if remaining.is_empty() {
            None
        } else {
            self.start = self.input.len();
            Some(InlineEvent::Text(remaining))
        }
    }
}

#[derive(Debug)]
struct CustomCharsIter<'a> {
    chars: Peekable<CharIndices<'a>>,
    current: Option<(usize, char)>,
    previous: Option<char>,
}

impl<'a> CustomCharsIter<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            current: None,
            previous: None,
        }
    }

    const fn previous(&self) -> Option<char> {
        self.previous
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied().map(|(_, c)| c)
    }

    fn next_if(&mut self, f: impl Fn(char) -> bool) -> Option<(usize, char)> {
        if let Some(p) = self.peek() {
            if f(p) {
                return self.next();
            }
        }
        None
    }

    fn next_if_eq(&mut self, c: char) -> Option<(usize, char)> {
        self.next_if(|n| n == c)
    }

    fn find(&mut self, c: char) -> Option<(usize, char)> {
        self.find_by(|n| n == c)
    }

    fn find_by(&mut self, f: impl Fn(char) -> bool) -> Option<(usize, char)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if f(n) {
                return Some((i, n));
            }
        }
    }

    fn find_with_previous(
        &mut self,
        c: char,
        prev_func: impl Fn(char) -> bool,
    ) -> Option<(usize, char)> {
        self.find_by_with_previous(|n| n == c, prev_func)
    }

    fn find_by_with_previous(
        &mut self,
        next_func: impl Fn(char) -> bool,
        prev_func: impl Fn(char) -> bool,
    ) -> Option<(usize, char)> {
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

    fn find_consecutive(&mut self, c: char, n: usize) -> Option<(usize, char)> {
        self.find_consecutive_by(|s| s == c, n)
    }

    fn find_consecutive_by(&mut self, f: impl Fn(char) -> bool, n: usize) -> Option<(usize, char)> {
        match n {
            0 => None,
            1 => self.find_by(f),
            _ => loop {
                if self.find_by(&f).is_none() {
                    return None;
                };

                let mut count = 1;
                while let Some((i, c)) = self.next_if(&f) {
                    count += 1;
                    if count == n {
                        return Some((i, c));
                    }
                }
            },
        }
    }

    fn count_consecutive(&mut self, c: char, max: usize) -> usize {
        self.count_consecutive_by(|n| n == c, max)
    }

    fn count_consecutive_by(&mut self, c: impl Fn(char) -> bool, max: usize) -> usize {
        if max == 0 {
            return 0;
        }

        let mut count = 0;
        while self.next_if(&c).is_some() {
            count += 1;
            if count == max {
                break;
            }
        }
        count
    }
}

impl<'a> Iterator for CustomCharsIter<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.chars.next();

        if next.is_none() {
            return None;
        }

        self.previous = self.current.map(|(_, c)| c);
        self.current = next;
        next
    }
}

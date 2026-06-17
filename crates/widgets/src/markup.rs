use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use ratatui::{crossterm::event::KeyCode, prelude::*};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    ansi::{AnsiEvent, AnsiParser, AnsiTag, AnsiWriter},
    kitty_graphics::{Dimensions, KittyGraphics, ResizeMode},
    text_segment::TextSegment,
};

pub struct Markup {
    items: Vec<Item>,
    ansi: AnsiWriter,
    wrapped_ansi: utils::Formatter,
    code_highlighter: CodeHighlighter,
    text_segment: TextSegment,
    scroll: u16,
    desired_scroll: Option<ScrollMove>,
    total_lines: u16,
    max_items: Option<usize>,
    image_id_start: u32,
    image_id_counter: u32,
    image_has_rendered: bool,
    area: Rect,
    hash: u64,
}

pub enum ScrollMove {
    Up,
    Down,
    PageUp,
    PageDown,
    Start,
    End,
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
    ImageDescription {
        text: Range<usize>,
    },
    Break,
    EmptyLine,
}

#[derive(Debug, Clone, Copy)]
pub enum MarkupItem {
    Paragraph,
    ListItem,
    Code,
    Image,
    ImageDescription,
    Break,
    EmptyLine,
}

impl Markup {
    pub fn new(syntax_highlight_theme: &'static str) -> Self {
        Self {
            items: Vec::new(),
            ansi: AnsiWriter::new(),
            wrapped_ansi: utils::Formatter::new(),
            code_highlighter: CodeHighlighter::new(syntax_highlight_theme),
            text_segment: TextSegment::new(),
            scroll: 0,
            desired_scroll: None,
            total_lines: 0,
            max_items: None,
            image_id_start: 90,
            image_id_counter: 0,
            image_has_rendered: false,
            area: Rect::ZERO,
            hash: 0,
        }
    }

    pub const fn scroll_index(&self) -> u16 {
        self.scroll
    }

    pub const fn set_desired_scroll(&mut self, sm: ScrollMove) -> &mut Self {
        self.desired_scroll = Some(sm);
        self
    }

    pub const fn set_max_items(&mut self, max: Option<usize>) -> &mut Self {
        self.max_items = max;
        self
    }

    pub fn input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Down => self.scroll(ScrollMove::Down),
            KeyCode::Up => self.scroll(ScrollMove::Up),
            KeyCode::PageDown => self.scroll(ScrollMove::PageDown),
            KeyCode::PageUp => self.scroll(ScrollMove::PageUp),
            KeyCode::Home => self.scroll(ScrollMove::Start),
            KeyCode::End => self.scroll(ScrollMove::End),
            _ => false,
        }
    }

    pub fn scroll(&mut self, sm: ScrollMove) -> bool {
        let old_scroll = self.scroll;

        self.scroll = match sm {
            ScrollMove::Up => self.scroll.saturating_sub(1),
            ScrollMove::Down => {
                (self.scroll + 1).min(self.total_lines.saturating_sub(self.area.height))
            }
            ScrollMove::PageUp => self.scroll.saturating_sub(self.area.height),
            ScrollMove::PageDown => (self.scroll + self.area.height)
                .min(self.total_lines.saturating_sub(self.area.height)),
            ScrollMove::Start => 0,
            ScrollMove::End => self.total_lines.saturating_sub(self.area.height),
        };

        self.scroll != old_scroll
    }

    pub fn render(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        text: &str,
        kitty: &mut KittyGraphics,
    ) {
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

        fn render_ansi_text(
            area: &mut Rect,
            buf: &mut Buffer,
            text: &str,
            current_line: &mut u16,
            viewport_top: u16,
            viewport_bot: u16,
            text_segment: &mut TextSegment,
            alignment: Alignment,
        ) {
            let mut style = Style::new();
            text_segment.set_alignment(alignment);

            for line in text.lines() {
                if is_in_viewport(*current_line, viewport_top, viewport_bot) {
                    for event in AnsiParser::new(line) {
                        match event {
                            AnsiEvent::Text(s) => {
                                text_segment.push_str(s, style);
                            }
                            AnsiEvent::Tag(tag) => match tag {
                                AnsiTag::Reset => {
                                    style = Style::new();
                                }
                                AnsiTag::Bold => {
                                    style.add_modifier.insert(Modifier::BOLD);
                                }
                                AnsiTag::Italic => {
                                    style.add_modifier.insert(Modifier::ITALIC);
                                }
                                AnsiTag::Reverse => {
                                    style.add_modifier.insert(Modifier::REVERSED);
                                }
                                AnsiTag::NotBold => {
                                    style.add_modifier.remove(Modifier::BOLD);
                                }
                                AnsiTag::NotItalic => {
                                    style.add_modifier.remove(Modifier::ITALIC);
                                }
                                AnsiTag::NotReverse => {
                                    style.add_modifier.remove(Modifier::REVERSED);
                                }
                                AnsiTag::FgTrueColor(r, g, b) => {
                                    style.fg = Some(Color::Rgb(r, g, b));
                                }
                                _ => {}
                            },
                        }
                    }
                    text_segment.render(*area, buf);
                    text_segment.clear();

                    area.y += 1;
                    area.height = area.height.saturating_sub(1);
                }

                *current_line += 1;
            }
        }

        // Update scroll
        self.total_lines = self.compute_total_lines(kitty);
        if let Some(sm) = self.desired_scroll.take() {
            self.scroll(sm);
        } else {
            self.scroll = self
                .scroll
                .min(self.total_lines.saturating_sub(area.height));
        }

        // Setup
        let top_y = area.y;
        let viewport_top = self.scroll;
        let viewport_bot = self.scroll + area.height;
        let mut current_line = 0;

        // Render
        for item in self.items.iter().cloned().take(self.max_items()) {
            if area.height == 0 {
                break;
            }

            match item {
                Item::Paragraph { text, alignment } => {
                    render_ansi_text(
                        &mut area,
                        buf,
                        self.wrapped_ansi.slice(text),
                        &mut current_line,
                        viewport_top,
                        viewport_bot,
                        &mut self.text_segment,
                        alignment,
                    );
                }
                Item::ListItem { text } => {
                    render_ansi_text(
                        &mut area,
                        buf,
                        self.wrapped_ansi.slice(text),
                        &mut current_line,
                        viewport_top,
                        viewport_bot,
                        &mut self.text_segment,
                        Alignment::Left,
                    );
                }
                Item::Code { text, .. } => {
                    render_ansi_text(
                        &mut area,
                        buf,
                        self.wrapped_ansi.slice(text),
                        &mut current_line,
                        viewport_top,
                        viewport_bot,
                        &mut self.text_segment,
                        Alignment::Left,
                    );
                }
                Item::Image { id, dims } => {
                    let max_width = kitty.width(area.width);
                    let resized_dims = KittyGraphics::resize(dims, dims.with_width(max_width));
                    let resized_area = kitty.area(resized_dims);

                    if is_in_viewport(
                        current_line,
                        viewport_top.saturating_sub(resized_area.rows - 1),
                        viewport_bot,
                    ) {
                        let is_at_top = area.y == top_y;
                        let rows_outside_top = if is_at_top {
                            current_line.abs_diff(self.scroll)
                        } else {
                            0
                        };
                        let image_rows = (resized_area.rows - rows_outside_top).min(area.height);
                        let image_area = Rect {
                            height: image_rows,
                            ..area
                        };
                        kitty.render(
                            image_area,
                            buf,
                            id,
                            dims,
                            ResizeMode::FitWidthCropHeight { rows_outside_top },
                            crate::utils::Alignment::CenterHorizontal,
                        );
                        self.image_has_rendered = true;

                        area.y += image_rows;
                        area.height = area.height.saturating_sub(image_rows);
                    }

                    current_line += resized_area.rows;
                }
                Item::ImageDescription { text } => {
                    render_ansi_text(
                        &mut area,
                        buf,
                        self.wrapped_ansi.slice(text),
                        &mut current_line,
                        viewport_top,
                        viewport_bot,
                        &mut self.text_segment,
                        Alignment::Center,
                    );
                }
                Item::Break => {
                    if is_in_viewport(current_line, viewport_top, viewport_bot) {
                        let half = area.width / 2;
                        let mut x = area.x + half / 2;
                        for _ in 0..half {
                            (x, _) = buf.set_stringn(x, area.y, "—", 1, Style::new());
                        }
                        area.y += 1;
                        area.height = area.height.saturating_sub(1);
                    }

                    current_line += 1;
                }
                Item::EmptyLine => {
                    if is_in_viewport(current_line, viewport_top, viewport_bot) {
                        area.y += 1;
                        area.height = area.height.saturating_sub(1);
                    }

                    current_line += 1;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.wrapped_ansi.clear();
        self.scroll = 0;
        self.desired_scroll = None;
        self.area = Rect::ZERO;
        self.hash = 0;
        self.total_lines = 0;
    }

    pub fn delete_images(&mut self, kitty: &KittyGraphics) -> std::io::Result<()> {
        if self.image_has_rendered {
            kitty.delete_range(
                self.image_id_start,
                self.image_id_start + self.image_id_counter.saturating_sub(1),
            )?;
            self.image_has_rendered = false;
        }
        Ok(())
    }

    pub fn parse_items(markup: &str, v: &mut Vec<MarkupItem>) {
        for (block, _) in BlockParser::new(markup) {
            match block {
                BlockElement::Paragraph { .. } => {
                    v.push(MarkupItem::Paragraph);
                }
                BlockElement::List { items } => {
                    for _ in items {
                        v.push(MarkupItem::ListItem);
                    }
                }
                BlockElement::Code { .. } => {
                    v.push(MarkupItem::Code);
                }
                BlockElement::Image { description, .. } => {
                    v.push(MarkupItem::Image);
                    if !description.is_empty() {
                        v.push(MarkupItem::ImageDescription);
                    }
                }
                BlockElement::Comment { _text } => continue,
                BlockElement::Break => {
                    v.push(MarkupItem::Break);
                }
            }
            v.push(MarkupItem::EmptyLine);
        }
        v.pop();
    }

    fn compute(&mut self, text: &str, width: u16, kitty: &mut KittyGraphics) {
        self.items.clear();
        self.wrapped_ansi.clear();
        self.image_id_counter = 0;

        for (block, _) in BlockParser::new(text) {
            match block {
                BlockElement::Paragraph { text, alignment } => {
                    let range = self.parse_text(text, width, None);
                    self.items.push(Item::Paragraph {
                        text: range,
                        alignment,
                    });
                }
                BlockElement::List { items } => {
                    for item in items {
                        let range =
                            self.parse_text(item, width.saturating_sub(4), Some(("  • ", "    ")));
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

                    let (lang_range, code_range) =
                        self.wrapped_ansi.push_str2(language, self.ansi.as_str());
                    self.items.push(Item::Code {
                        _language: lang_range,
                        text: code_range,
                    });
                    self.ansi.clear();
                }
                BlockElement::Image { description, path } => {
                    kitty.load(path).unwrap();
                    let id = self.image_id_start + self.image_id_counter;
                    let dims = kitty.encode(id).unwrap();
                    self.items.push(Item::Image { id, dims });
                    self.image_id_counter += 1;

                    if !description.is_empty() {
                        let range = self.parse_text(description, width, None);
                        self.items.push(Item::ImageDescription { text: range });
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

    fn parse_text(&mut self, text: &str, width: u16, indent: Option<(&str, &str)>) -> Range<usize> {
        // Convert markup to ansi
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

        // Store result and use later with returned range
        let range = match indent {
            Some((first_indent, other_indent)) => {
                let start = self.wrapped_ansi.len();
                for (i, line) in self.ansi.as_str().lines().enumerate() {
                    let indent = if i == 0 { first_indent } else { other_indent };
                    self.wrapped_ansi.extend([indent, line, "\n"]);
                }
                let end = self.wrapped_ansi.len();
                start..end
            }
            None => self.wrapped_ansi.push_str(self.ansi.as_str()),
        };
        self.ansi.clear();
        range
    }

    const fn max_items(&self) -> usize {
        match self.max_items {
            Some(max) => max,
            None => self.items.len(),
        }
    }

    fn compute_total_lines(&self, kitty: &KittyGraphics) -> u16 {
        let mut total_lines = 0;

        for item in self.items.iter().cloned().take(self.max_items()) {
            match item {
                Item::Paragraph { text, .. } => {
                    total_lines += self.wrapped_ansi.slice(text).lines().count() as u16;
                }
                Item::ListItem { text } => {
                    total_lines += self.wrapped_ansi.slice(text).lines().count() as u16;
                }
                Item::Code { text, .. } => {
                    total_lines += self.wrapped_ansi.slice(text).lines().count() as u16;
                }
                Item::Image { dims, .. } => {
                    let max_width = kitty.width(self.area.width);
                    let resized_dims = KittyGraphics::resize(dims, dims.with_width(max_width));
                    let resized_area = kitty.area(resized_dims);
                    total_lines += resized_area.rows;
                }
                Item::ImageDescription { text } => {
                    total_lines += self.wrapped_ansi.slice(text).lines().count() as u16;
                }
                Item::Break => {
                    total_lines += 1;
                }
                Item::EmptyLine => {
                    total_lines += 1;
                }
            }
        }

        total_lines
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
    graphemes: utils::PeekableGraphemesPrevious<'a>,
}

impl<'a> BlockParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            graphemes: utils::PeekableGraphemesPrevious::new(input),
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
        let Some((descr_start, "[")) = self.graphemes.next() else {
            return self.parse_paragraph(start, Alignment::Left);
        };
        let Some((descr_end, _)) = self.graphemes.find("]") else {
            return self.parse_paragraph(start, Alignment::Left);
        };

        let Some((path_start, "(")) = self.graphemes.next() else {
            return self.parse_paragraph(start, Alignment::Left);
        };
        let Some((path_end, _)) = self.graphemes.find(")") else {
            return self.parse_paragraph(start, Alignment::Left);
        };

        let description = self.input[descr_start + 1..descr_end].trim();
        let path = self.input[path_start + 1..path_end].trim();

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
    graphemes: utils::PeekableGraphemesPrevious<'a>,
    start: usize,
}

impl<'a> ListItems<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            graphemes: utils::PeekableGraphemesPrevious::new(text),
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

struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme: &'static str,
}

impl CodeHighlighter {
    fn new(theme: &'static str) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme,
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

        let mut highlighter = HighlightLines::new(syntax, &self.theme_set.themes[self.theme]);
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
    chars: utils::PeekableCharsPrevious<'a>,
    start: usize,
    tag: Option<InlineTag>,
    bold: bool,
    italic: bool,
}

impl<'a> InlineParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: utils::PeekableCharsPrevious::new(input),
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
                    self.chars.previous(),
                    self.chars.peek(),
                    InlineTag::BoldStart,
                    InlineTag::BoldEnd,
                    &mut self.bold,
                ),
                '_' => parse_tag(
                    c,
                    self.chars.previous(),
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

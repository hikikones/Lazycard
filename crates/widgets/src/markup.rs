use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Alignment, Rect, Size},
    style::Style,
};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use unicode_segmentation::UnicodeSegmentation;
use utils::Formatter;

use crate::{
    Scrollbar, ScrollbarColors,
    ansi::{AnsiParser, AnsiTag, AnsiWriter},
    kitty_graphics::{Dimensions, KittyGraphics, ResizeMode},
    text_segment::TextSegment,
};

pub struct Markup {
    markup_items: Vec<Item>,
    markup_buf: Formatter,
    items: Vec<MarkupRender>,
    ansi: AnsiWriter,
    buffer: String,
    wrapped_ansi: Formatter,
    code_highlighter: CodeHighlighter,
    text_segment: TextSegment,
    scroll: u16,
    desired_scroll: Option<ScrollMove>,
    max_lines: u16,
    total_lines: u16,
    scrollbar: Option<ScrollbarColors>,
    scroll_area: Option<Rect>,
    max_items: Option<usize>,
    image_id_start: u32,
    image_id_counter: u32,
    image_has_rendered: bool,
    assets_path: PathBuf,
    size: Size,
    area: Rect,
    hash: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollMove {
    Up,
    Down,
    PageUp,
    PageDown,
    Start,
    End,
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
    Image(ImageItem),
    ImageDescription {
        text: Range<usize>,
    },
    Break,
    EmptyLine,
}

#[derive(Debug, Clone)]
enum MarkupRender {
    Text {
        range: Range<usize>,
        alignment: Alignment,
    },
    Image {
        id: u32,
        dims: Dimensions,
    },
    Break,
    EmptyLine,
}

#[derive(Debug, Clone)]
enum ImageItem {
    Ok { id: u32, dims: Dimensions },
    Err { text: Range<usize> },
}

impl Markup {
    pub fn new(assets: PathBuf, theme: SyntaxHighlightTheme) -> Self {
        Self {
            markup_items: Vec::new(),
            markup_buf: Formatter::new(),
            items: Vec::new(),
            ansi: AnsiWriter::new(),
            buffer: String::new(),
            wrapped_ansi: Formatter::new(),
            code_highlighter: CodeHighlighter::new(theme),
            text_segment: TextSegment::new(),
            scroll: 0,
            desired_scroll: None,
            max_lines: 0,
            total_lines: 0,
            scrollbar: None,
            scroll_area: None,
            max_items: None,
            image_id_start: 90,
            image_id_counter: 0,
            image_has_rendered: false,
            assets_path: assets,
            size: Size::ZERO,
            area: Rect::ZERO,
            hash: 0,
        }
    }

    pub const fn with_scrollbar(mut self, colors: ScrollbarColors) -> Self {
        self.scrollbar = Some(colors);
        self
    }

    pub const fn scroll_index(&self) -> u16 {
        self.scroll
    }

    pub const fn set_desired_scroll(&mut self, sm: ScrollMove) -> &mut Self {
        self.desired_scroll = Some(sm);
        self
    }

    pub const fn set_scrollbar(&mut self, colors: ScrollbarColors) -> &mut Self {
        self.scrollbar = Some(colors);
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
                (self.scroll + 1).min(self.max_lines.saturating_sub(self.size.height))
            }
            ScrollMove::PageUp => self.scroll.saturating_sub(self.size.height),
            ScrollMove::PageDown => (self.scroll + self.size.height)
                .min(self.max_lines.saturating_sub(self.size.height)),
            ScrollMove::Start => 0,
            ScrollMove::End => self.max_lines.saturating_sub(self.size.height),
        };

        self.scroll != old_scroll
    }

    pub fn render(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        markup: &str,
        kitty: &mut KittyGraphics,
    ) {
        if area.is_empty() {
            return;
        }

        let hash = utils::hash_fast(markup);
        if self.size != area.as_size() || self.hash != hash {
            self.size = area.as_size();
            self.area = area;
            self.hash = hash;
            self.scroll_area = None;

            self.parse_and_load(markup, kitty);
            self.process_layout(area.width, kitty);

            if self.scrollbar.is_some()
                && Scrollbar::is_scrollable(self.total_lines as usize, area.as_size())
            {
                let scroll_area = Scrollbar::make_scroll_area(&mut area);
                self.process_layout(area.width, kitty);
                self.area = area;
                self.scroll_area = Some(scroll_area);
            }
        }

        const fn is_in_viewport(curr_line: u16, top: u16, bot: u16) -> bool {
            curr_line >= top && curr_line < bot
        }

        let mut area = self.area;

        // Update scroll
        self.max_lines = self.compute_lines(area.width, self.max_items(), kitty);
        if let Some(sm) = self.desired_scroll.take() {
            self.scroll(sm);
        } else {
            self.scroll = self.scroll.min(self.max_lines.saturating_sub(area.height));
        }

        // Scrollbar
        if let Some((scroll_area, colors)) = self.scroll_area.zip(self.scrollbar) {
            Scrollbar::new().with_colors(colors).render(
                scroll_area,
                buf,
                self.scroll as usize,
                self.total_lines as usize,
            );
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
                MarkupRender::Text { range, alignment } => {
                    let mut ansi_parser = AnsiParser::new("").with_style();
                    self.text_segment.set_alignment(alignment);

                    for line in self.wrapped_ansi.slice(range).lines() {
                        let is_in_viewport =
                            is_in_viewport(current_line, viewport_top, viewport_bot);

                        for (s, style) in ansi_parser.continue_with(line) {
                            if is_in_viewport {
                                self.text_segment.push_str(s, style);
                            }
                        }

                        if is_in_viewport {
                            self.text_segment.render(area, buf);
                            self.text_segment.clear();

                            area.y += 1;
                            area.height = area.height.saturating_sub(1);
                        }

                        current_line += 1;
                    }
                }
                MarkupRender::Image { id, dims } => {
                    let max_width = kitty.width(area.width);
                    let resized_dims = KittyGraphics::resize(dims, dims.with_width(max_width));
                    let resized_rows = kitty.rows(resized_dims.height);

                    if is_in_viewport(
                        current_line,
                        viewport_top.saturating_sub(resized_rows - 1),
                        viewport_bot,
                    ) {
                        let is_at_top = area.y == top_y;
                        let rows_outside_top = if is_at_top {
                            current_line.abs_diff(self.scroll)
                        } else {
                            0
                        };
                        let image_rows = (resized_rows - rows_outside_top).min(area.height);
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

                    current_line += resized_rows;
                }
                MarkupRender::Break => {
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
                MarkupRender::EmptyLine => {
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
        self.markup_items.clear();
        self.markup_buf.clear();
        self.buffer.clear();
        self.items.clear();
        self.wrapped_ansi.clear();
        self.scroll = 0;
        self.desired_scroll = None;
        self.max_items = None;
        self.max_lines = 0;
        self.total_lines = 0;
        self.scroll_area = None;
        self.size = Size::ZERO;
        self.area = Rect::ZERO;
        self.hash = 0;
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

    fn parse_and_load(&mut self, markup: &str, kitty: &mut KittyGraphics) {
        self.markup_items.clear();
        self.markup_buf.clear();
        self.buffer.clear();
        self.image_id_counter = 0;

        // Convert tabs to spaces before we load markup
        let markup = if markup.contains('\t') {
            self.buffer.extend(
                markup
                    .graphemes(true)
                    .map(|g| if g == "\t" { "    " } else { g }),
            );
            self.buffer.as_str()
        } else {
            markup
        };

        // Load markup
        for (block, _) in BlockParser::new(markup) {
            match block {
                BlockElement::Paragraph { text, alignment } => {
                    self.markup_items.push(Item::Paragraph {
                        text: self.markup_buf.push_str(text),
                        alignment,
                    });
                }
                BlockElement::List { items } => {
                    for item in items {
                        self.markup_items.push(Item::ListItem {
                            text: self.markup_buf.push_str(item),
                        });
                    }
                }
                BlockElement::Code { language, text } => {
                    self.markup_items.push(Item::Code {
                        text: self.markup_buf.push_str(text),
                        _language: self.markup_buf.push_str(language),
                    });
                }
                BlockElement::Image { description, path } => {
                    let image_path = Path::new(path)
                        .file_name()
                        .map(|name| self.assets_path.join(name));

                    fn load_and_encode_image(
                        path: Option<PathBuf>,
                        id: u32,
                        kitty: &mut KittyGraphics,
                    ) -> Result<Dimensions, String> {
                        let Some(path) = path else {
                            return Err(String::from("No image filename found"));
                        };
                        kitty.load(&path).map_err(|err| {
                            format!(
                                "Failed to load image\n\"{}\"\ndue to\n\"{}\"",
                                path.display(),
                                err
                            )
                        })?;
                        let dims = kitty.encode(id).map_err(|err| {
                            format!(
                                "Failed to encode image\n\"{}\"\ndue to\n\"{}\"",
                                path.display(),
                                err
                            )
                        })?;
                        Ok(dims)
                    }

                    let id = self.image_id_start + self.image_id_counter;
                    match load_and_encode_image(image_path, id, kitty) {
                        Ok(dims) => {
                            self.markup_items
                                .push(Item::Image(ImageItem::Ok { id, dims }));
                            self.image_id_counter += 1;
                        }
                        Err(err) => {
                            self.markup_items.push(Item::Image(ImageItem::Err {
                                text: self.markup_buf.extend(["ERROR\n", err.as_str()]),
                            }));
                        }
                    }

                    if !description.is_empty() {
                        self.markup_items.push(Item::ImageDescription {
                            text: self.markup_buf.push_str(description),
                        });
                    }
                }
                BlockElement::Break => {
                    self.markup_items.push(Item::Break);
                }
                BlockElement::Comment { .. } => continue,
            }

            // Add empty line between each block element
            self.markup_items.push(Item::EmptyLine);
        }

        // Remove last empty line
        self.markup_items.pop();
    }

    fn process_layout(&mut self, width: u16, kitty: &KittyGraphics) {
        self.items.clear();
        self.wrapped_ansi.clear();

        // Process loaded markup
        self.items
            .extend(self.markup_items.iter().cloned().map(|item| match item {
                Item::Paragraph { text, alignment } => MarkupRender::Text {
                    range: markup_to_wrapped_ansi(
                        self.markup_buf.slice(text),
                        width,
                        None,
                        &mut self.ansi,
                        &mut self.wrapped_ansi,
                    ),
                    alignment,
                },
                Item::ListItem { text } => MarkupRender::Text {
                    range: markup_to_wrapped_ansi(
                        self.markup_buf.slice(text),
                        width.saturating_sub(4),
                        Some(("  • ", "    ")),
                        &mut self.ansi,
                        &mut self.wrapped_ansi,
                    ),
                    alignment: Alignment::Left,
                },
                Item::Code { text, _language } => {
                    let lang = self.markup_buf.slice(_language);
                    let code = self.markup_buf.slice(text);

                    self.code_highlighter
                        .highlight(lang, code, |span, color| match color {
                            Some((r, g, b)) => {
                                self.ansi.push_tag(AnsiTag::FgTrueColor(r, g, b));
                                self.ansi.push_str(span);
                                self.ansi.push_tag(AnsiTag::Reset);
                            }
                            None => {
                                self.ansi.push_str(span);
                            }
                        });

                    let range = self.wrapped_ansi.push_str(self.ansi.as_str());
                    self.ansi.clear();

                    MarkupRender::Text {
                        range,
                        alignment: Alignment::Left,
                    }
                }
                Item::Image(image) => match image {
                    ImageItem::Ok { id, dims } => MarkupRender::Image { id, dims },
                    ImageItem::Err { text } => MarkupRender::Text {
                        range: markup_to_wrapped_ansi(
                            self.markup_buf.slice(text),
                            width,
                            None,
                            &mut self.ansi,
                            &mut self.wrapped_ansi,
                        ),
                        alignment: Alignment::Center,
                    },
                },
                Item::ImageDescription { text } => MarkupRender::Text {
                    range: markup_to_wrapped_ansi(
                        self.markup_buf.slice(text),
                        width,
                        None,
                        &mut self.ansi,
                        &mut self.wrapped_ansi,
                    ),
                    alignment: Alignment::Center,
                },
                Item::Break => MarkupRender::Break,
                Item::EmptyLine => MarkupRender::EmptyLine,
            }));

        // Compute total lines
        self.total_lines = self.compute_lines(width, self.items.len(), kitty);
    }

    fn compute_lines(&self, width: u16, max_items: usize, kitty: &KittyGraphics) -> u16 {
        self.items
            .iter()
            .cloned()
            .take(max_items)
            .map(|item| match item {
                MarkupRender::Text { range, .. } => {
                    self.wrapped_ansi.slice(range).lines().count() as u16
                }
                MarkupRender::Image { dims, .. } => {
                    let max_width = kitty.width(width);
                    let resized_dims = KittyGraphics::resize(dims, dims.with_width(max_width));
                    kitty.rows(resized_dims.height)
                }
                MarkupRender::Break | MarkupRender::EmptyLine => 1,
            })
            .sum()
    }

    const fn max_items(&self) -> usize {
        match self.max_items {
            Some(max) => max,
            None => self.items.len(),
        }
    }
}

fn markup_to_wrapped_ansi(
    markup: &str,
    width: u16,
    indent: Option<(&str, &str)>,
    writer: &mut AnsiWriter,
    storage: &mut Formatter,
) -> Range<usize> {
    // Convert markup to ansi
    for event in InlineParser::new(markup) {
        match event {
            InlineEvent::Text(s) => writer.push_str(s),
            InlineEvent::Tag(tag) => writer.push_tag(tag.into_ansi()),
        }
    }

    // Break text into lines using textwrap which ignores ansi codes
    writer.textwrap(width);

    // Store result for later usage
    let range = match indent {
        Some((first_indent, other_indent)) => {
            let start = storage.len();
            for (i, line) in writer.as_str().lines().enumerate() {
                let indent = if i == 0 { first_indent } else { other_indent };
                storage.extend([indent, line, "\n"]);
            }
            let end = storage.len();
            start..end
        }
        None => storage.push_str(writer.as_str()),
    };
    writer.clear();
    range
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

#[derive(Debug, Clone, Copy)]
pub enum SyntaxHighlightTheme {
    Base16OceanDark,
    Base16OceanLight,
    Base16MochaDark,
    Base16EightiesDark,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

impl SyntaxHighlightTheme {
    const fn as_str(self) -> &'static str {
        match self {
            SyntaxHighlightTheme::Base16OceanDark => "base16-ocean.dark",
            SyntaxHighlightTheme::Base16OceanLight => "base16-ocean.light",
            SyntaxHighlightTheme::Base16MochaDark => "base16-mocha.dark",
            SyntaxHighlightTheme::Base16EightiesDark => "base16-eighties.dark",
            SyntaxHighlightTheme::InspiredGitHub => "InspiredGitHub",
            SyntaxHighlightTheme::SolarizedDark => "Solarized (dark)",
            SyntaxHighlightTheme::SolarizedLight => "Solarized (light)",
        }
    }
}

struct CodeHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme: &'static str,
}

impl CodeHighlighter {
    fn new(theme: SyntaxHighlightTheme) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme: theme.as_str(),
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

impl InlineTag {
    const fn into_ansi(self) -> AnsiTag {
        match self {
            InlineTag::BoldStart => AnsiTag::Bold,
            InlineTag::BoldEnd => AnsiTag::NotBold,
            InlineTag::ItalicStart => AnsiTag::Italic,
            InlineTag::ItalicEnd => AnsiTag::NotItalic,
        }
    }
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

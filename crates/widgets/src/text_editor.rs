use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Position, Rect, Size},
    style::{Color, Style},
};

use crate::AnsiWriter;

// TODO: Use ansi codes for syntax highlighting.

pub struct TextEditor {
    input: String,
    cursor: usize,
    selector: Option<usize>,
    placeholder: &'static str,
    ansi: AnsiWriter,
    lines: Vec<VisualLine>,
    preferred_column: u16,
    scroll: u16,
    disabled: bool,
    margin_top: usize,
    margin_bottom: usize,
    colors: TextEditorColors,
    last_size: Size,
    last_hash: u64,
}

pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Start,
    End,
}

pub enum CursorDelete {
    Forward,
    Back,
    Selection,
}

pub struct TextEditorColors {
    pub normal: Color,
    pub cursor: Color,
    pub selector: Color,
    pub placeholder: Color,
    pub disabled: Color,
}

impl TextEditorColors {
    pub const fn new() -> Self {
        Self {
            normal: Color::Reset,
            cursor: Color::White,
            selector: Color::DarkGray,
            placeholder: Color::DarkGray,
            disabled: Color::DarkGray,
        }
    }

    pub const fn all(color: Color) -> Self {
        Self {
            normal: color,
            cursor: color,
            selector: color,
            placeholder: color,
            disabled: color,
        }
    }
}

impl Default for TextEditorColors {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEditor {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            selector: None,
            placeholder: "",
            ansi: AnsiWriter::new(),
            lines: Vec::new(),
            preferred_column: 0,
            scroll: 0,
            disabled: false,
            margin_top: 0,
            margin_bottom: 0,
            colors: TextEditorColors::new(),
            last_size: Size::ZERO,
            last_hash: 0,
        }
    }

    pub const fn with_placeholder(mut self, s: &'static str) -> Self {
        self.placeholder = s;
        self
    }

    pub const fn with_colors(mut self, colors: TextEditorColors) -> Self {
        self.set_colors(colors);
        self
    }

    pub const fn with_margins(mut self, top: usize, bottom: usize) -> Self {
        self.margin_top = top;
        self.margin_bottom = bottom;
        self
    }

    pub const fn with_disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub const fn set_colors(&mut self, colors: TextEditorColors) -> &mut Self {
        self.colors = colors;
        self
    }

    pub const fn set_disabled(&mut self, value: bool) -> &mut Self {
        self.disabled = value;
        self
    }

    pub const fn toggle_disabled(&mut self) -> &mut Self {
        self.set_disabled(!self.disabled)
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.input.as_str()
    }

    pub fn input(&mut self, key_pressed: KeyCode, key_modifiers: KeyModifiers) -> bool {
        if self.disabled {
            return false;
        }

        let ctrl = key_modifiers.contains(KeyModifiers::CONTROL);
        let shift = key_modifiers.contains(KeyModifiers::SHIFT);

        match key_pressed {
            KeyCode::Right => return self.move_cursor(CursorMove::Forward, shift),
            KeyCode::Left => return self.move_cursor(CursorMove::Back, shift),
            KeyCode::Up => return self.move_cursor(CursorMove::Up, shift),
            KeyCode::Down => return self.move_cursor(CursorMove::Down, shift),
            KeyCode::Home => return self.move_cursor(CursorMove::Start, shift),
            KeyCode::End => return self.move_cursor(CursorMove::End, shift),
            KeyCode::Backspace => return self.delete(CursorDelete::Back),
            KeyCode::Delete => return self.delete(CursorDelete::Forward),
            KeyCode::Enter => {
                self.push_newline();
                return true;
            }
            KeyCode::Char(c) => match c {
                'a' => {
                    if ctrl {
                        return self.select_all();
                    }

                    self.push_char(c);
                    return true;
                }
                _ => {
                    self.push_char(c);
                    return true;
                }
            },
            _ => false,
        }
    }

    pub fn push_char(&mut self, c: char) {
        if let Some(start) = self.selector.take() {
            self.delete_selection(start);
        }
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn push_str(&mut self, s: &str) {
        if let Some(start) = self.selector.take() {
            self.delete_selection(start);
        }
        self.input.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    pub fn push_newline(&mut self) {
        #[cfg(target_os = "windows")]
        self.push_str("\r\n");
        #[cfg(not(target_os = "windows"))]
        self.push_char('\n');
    }

    pub fn move_cursor(&mut self, cm: CursorMove, shift: bool) -> bool {
        let (old_cursor, old_selector) = (self.cursor, self.selector);

        if shift {
            if self.selector.is_none() {
                self.selector = Some(self.cursor);
            }
        } else {
            self.selector = None;
        }

        match cm {
            CursorMove::Forward => {
                if let Some(g) = graphemes(&self.input[self.cursor..]).next() {
                    self.cursor += g.len();
                    self.preferred_column = self.cursor_position().x;
                }
            }
            CursorMove::Back => {
                if let Some(g) = graphemes(&self.input[..self.cursor]).next_back() {
                    self.cursor -= g.len();
                    self.preferred_column = self.cursor_position().x;
                }
            }
            CursorMove::Up => {
                let row = self.cursor_row();
                if row == 0 {
                    self.cursor = 0;
                    self.preferred_column = 0;
                } else {
                    self.cursor = self.column_to_index(row - 1, self.preferred_column);
                }
            }
            CursorMove::Down => {
                let row = self.cursor_row();
                if row + 1 >= self.lines.len() as u16 {
                    self.cursor = self.input.len();
                    self.preferred_column = self.cursor_position().x;
                } else {
                    self.cursor = self.column_to_index(row + 1, self.preferred_column);
                }
            }
            CursorMove::Start => {
                self.cursor = 0;
                self.preferred_column = 0;
            }
            CursorMove::End => {
                self.cursor = self.input.len();
                self.preferred_column = self.cursor_position().x;
            }
        }

        self.selector.take_if(|s| *s == self.cursor);

        self.cursor != old_cursor || self.selector != old_selector
    }

    pub fn select_all(&mut self) -> bool {
        let (old_cursor, old_selector) = (self.cursor, self.selector);

        self.cursor = self.input.len();
        self.selector = Some(0);
        self.preferred_column = self.cursor_position().x;

        self.cursor != old_cursor || self.selector != old_selector
    }

    pub fn delete(&mut self, cd: CursorDelete) -> bool {
        match cd {
            CursorDelete::Forward => match self.selector.take() {
                Some(selector) => self.delete_selection(selector),
                None => match graphemes(&self.input[self.cursor..]).next() {
                    Some(g) => {
                        self.input
                            .replace_range(self.cursor..self.cursor + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::Back => match self.selector.take() {
                Some(selector) => self.delete_selection(selector),
                None => match graphemes(&self.input[..self.cursor]).next_back() {
                    Some(g) => {
                        self.cursor -= g.len();
                        self.input
                            .replace_range(self.cursor..self.cursor + g.len(), "");
                        true
                    }
                    None => false,
                },
            },
            CursorDelete::Selection => match self.selector.take() {
                Some(selector) => self.delete_selection(selector),
                None => false,
            },
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() || buf.cell(area.as_position()).is_none() {
            return;
        }

        let last_size = self.last_size;

        // Disabled
        if self.disabled {
            if self.input.is_empty() {
                buf.set_stringn(
                    area.x,
                    area.y,
                    self.placeholder,
                    area.width as usize,
                    self.colors.disabled,
                );
            } else {
                self.process_input(last_size, area.as_size());
                self.update_scroll(last_size.height, area.height);
                self.render_text(area, buf, self.colors.disabled);
            }
            return;
        }

        // Placeholder
        if self.input.is_empty() {
            let Rect { x, y, .. } = area;
            buf.set_stringn(
                x,
                y,
                self.placeholder,
                area.width as usize,
                self.colors.placeholder,
            );
            buf[(x, y)].set_style(Style::new().fg(self.colors.cursor).reversed());
            return;
        }

        // Render
        self.process_input(last_size, area.as_size());
        self.update_scroll(last_size.height, area.height);
        self.render_text(area, buf, self.colors.normal);
        self.render_selection(area, buf, self.colors.selector);
        self.render_cursor(area, buf, self.colors.cursor);
    }

    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.selector = None;
        self.ansi.clear();
        self.lines.clear();
        self.preferred_column = 0;
        self.scroll = 0;
        self.last_size = Size::ZERO;
        self.last_hash = 0;
    }

    fn selection(&self, selector: usize) -> Option<std::ops::Range<usize>> {
        use std::cmp::Ordering;

        match self.cursor.cmp(&selector) {
            Ordering::Less => Some(self.cursor..selector),
            Ordering::Greater => Some(selector..self.cursor),
            Ordering::Equal => None,
        }
    }

    fn try_selection(&self) -> Option<std::ops::Range<usize>> {
        self.selector.and_then(|selector| self.selection(selector))
    }

    fn delete_selection(&mut self, selector: usize) -> bool {
        let Some(range) = self.selection(selector) else {
            return false;
        };
        self.cursor = range.start;
        self.input.replace_range(range, "");
        true
    }

    fn render_text(&self, area: Rect, buf: &mut Buffer, color: Color) {
        let style = Style::new().fg(color);

        for (i, line) in self
            .lines
            .iter()
            .skip(self.scroll as usize)
            .take(area.height as usize)
            .enumerate()
        {
            let (mut x, y, mut width) = (area.x, area.y + i as u16, line.width);
            for g in graphemes(self.ansi.slice(line.range())).map(grapheme_render) {
                let (next_x, _) = buf.set_stringn(x, y, g, width as usize, style);
                width -= next_x - x;
                x = next_x;
            }
        }
    }

    fn render_selection(&self, area: Rect, buf: &mut Buffer, color: Color) {
        if let Some(range) = self.try_selection() {
            let start = self.index_to_position(range.start);
            let end = self.index_to_position(range.end);

            let style = Style::new().fg(color).reversed();

            for (i, line) in self
                .lines
                .iter()
                .enumerate()
                .skip(self.scroll as usize)
                .take(area.height as usize)
            {
                let i = i as u16;
                if i < start.y || i > end.y {
                    continue;
                }

                let col_start = if i == start.y { start.x } else { 0 };
                let col_end = if i == end.y { end.x } else { line.width };

                for col in col_start..col_end {
                    let x = area.x + col;
                    let y = area.y + i.saturating_sub(self.scroll);
                    match buf.cell_mut((x, y)) {
                        Some(cell) => {
                            cell.set_style(style);
                        }
                        None => break,
                    }
                }
            }
        }
    }

    fn render_cursor(&self, area: Rect, buf: &mut Buffer, color: Color) {
        let mut cpos = self.cursor_position() + area.as_position().into();
        cpos.y = cpos.y.saturating_sub(self.scroll);

        if let Some(cell) = buf.cell_mut(cpos) {
            cell.set_style(Style::new().fg(color).reversed());
        }
    }

    fn update_scroll(&mut self, last_height: u16, height: u16) {
        let scroll = if last_height != height {
            // Refresh scroll on window resize
            0
        } else {
            self.scroll
        };
        self.scroll = crate::Scrollbar::calculate_scroll_with_margins(
            self.lines.len(),
            height,
            self.index_to_row(self.cursor) as usize,
            scroll as usize,
            self.margin_top,
            self.margin_bottom,
            0,
        ) as u16;
    }

    fn process_input(&mut self, last_size: Size, size: Size) {
        let hash = utils::hash_fast(self.input.as_str());
        if last_size.width != size.width || self.last_hash != hash {
            self.last_size = size;
            self.last_hash = hash;
            self.relayout(size.width);
        }
    }

    fn relayout(&mut self, max_width: u16) {
        self.ansi.clear();
        self.lines.clear();

        self.ansi.push_str(self.input.as_str());
        self.ansi.textwrap(max_width);

        let mut start = 0;
        let mut column = 0;

        for (i, g) in grapheme_indices(self.ansi.as_str()) {
            if g.contains('\n') {
                self.lines
                    .push(VisualLine::new(self.ansi.as_str(), start..i + g.len()));

                start = i + g.len();
                column = 0;
                continue;
            }

            let width = grapheme_width(g);
            if column + width > max_width {
                self.lines
                    .push(VisualLine::new(self.ansi.as_str(), start..i));

                start = i;
                column = width;
            } else {
                column += width;
            }
        }

        self.lines
            .push(VisualLine::new(self.ansi.as_str(), start..self.ansi.len()));
        self.preferred_column = self.cursor_position().x;
    }

    fn cursor_row(&self) -> u16 {
        self.index_to_row(self.cursor)
    }

    fn cursor_position(&self) -> Position {
        self.index_to_position(self.cursor)
    }

    fn index_to_row(&self, index: usize) -> u16 {
        self.lines
            .partition_point(|line| line.end <= index)
            .min(self.lines.len().saturating_sub(1)) as u16
    }

    fn index_to_position(&self, index: usize) -> Position {
        let row = self.index_to_row(index);
        let line = &self.lines[row as usize];
        let text = self.ansi.slice(line.range());
        let col = text_width(&text[..index - line.start]);
        Position { x: col, y: row }
    }

    fn column_to_index(&self, row: u16, target: u16) -> usize {
        let line = &self.lines[row as usize];

        let mut column = 0;
        let mut index = line.start;

        for (i, g) in grapheme_indices(self.ansi.slice(line.range())) {
            if g.contains('\n') {
                break;
            }

            let width = grapheme_width(g);
            if column + width > target {
                break;
            }

            column += width;
            index = line.start + i + g.len();
        }

        index
    }
}

#[derive(Debug, Clone)]
struct VisualLine {
    start: usize,
    end: usize,
    width: u16,
}

impl VisualLine {
    fn new(text: &str, range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
            width: text_width(&text[range]),
        }
    }

    const fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

fn text_width(s: &str) -> u16 {
    graphemes(s).map(grapheme_width).sum()
}

fn grapheme_width(g: &str) -> u16 {
    match g {
        "\t" => 4,
        _ => unicode_width::UnicodeWidthStr::width(g) as u16,
    }
}

fn grapheme_render(g: &str) -> &str {
    match g {
        "\t" => "    ",
        _ => g,
    }
}

fn graphemes(s: &str) -> unicode_segmentation::Graphemes<'_> {
    use unicode_segmentation::UnicodeSegmentation;
    s.graphemes(true)
}

fn grapheme_indices(s: &str) -> unicode_segmentation::GraphemeIndices<'_> {
    use unicode_segmentation::UnicodeSegmentation;
    s.grapheme_indices(true)
}

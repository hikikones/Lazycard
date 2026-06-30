use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

pub struct Scrollbar {
    colors: ScrollbarColors,
}

impl Scrollbar {
    pub const fn new() -> Self {
        Self {
            colors: ScrollbarColors::new(Color::DarkGray, None),
        }
    }

    pub const fn with_colors(mut self, colors: ScrollbarColors) -> Self {
        self.set_colors(colors);
        self
    }

    pub const fn set_colors(&mut self, colors: ScrollbarColors) -> &mut Self {
        self.colors = colors;
        self
    }

    pub fn render(
        &self,
        vertical_line: Rect,
        buf: &mut Buffer,
        current_scroll: usize,
        total_items: usize,
    ) {
        let height = vertical_line.height as usize;
        if total_items == 0 || height == 0 {
            return;
        }

        let visible = height as f32 / total_items as f32;
        let size = ((visible * height as f32).round() as usize).max(1);
        let progress = (current_scroll as f32 / total_items.saturating_sub(height) as f32).min(1.0);
        let range = height.saturating_sub(size);
        let start = (progress * range as f32).round() as usize;
        let end = start + size;

        let thumb_style = Style::new().fg(self.colors.thumb);
        let Rect { x, mut y, .. } = vertical_line;

        match self.colors.track {
            // Render both track and thumb
            Some(track_color) => {
                let track_style = Style::new().fg(track_color);
                for i in 0..height {
                    match buf.cell_mut((x, y)) {
                        Some(cell) => {
                            let (symbol, style) = if i >= start && i < end {
                                ("┃", thumb_style)
                            } else {
                                ("│", track_style)
                            };
                            cell.set_symbol(symbol).set_style(style);
                        }
                        None => return,
                    }
                    y += 1;
                }
            }
            // Render only thumb
            None => {
                for i in 0..height {
                    match buf.cell_mut((x, y)) {
                        Some(cell) => {
                            if i >= start && i < end {
                                cell.set_symbol("│").set_style(thumb_style);
                            }
                        }
                        None => return,
                    }
                    y += 1;
                }
            }
        }
    }

    pub fn calculate_scroll(
        total_lines: usize,
        viewport_height: u16,
        current_index: usize,
        current_scroll: usize,
    ) -> usize {
        Self::calculate_scroll_with_margins(
            total_lines,
            viewport_height,
            current_index,
            current_scroll,
            0,
            0,
            0,
        )
    }

    pub fn calculate_scroll_with_margins(
        total_lines: usize,
        viewport_height: u16,
        current_index: usize,
        current_scroll: usize,
        margin_top: usize,
        margin_bottom: usize,
        padding_bottom: usize,
    ) -> usize {
        let height = viewport_height as usize;
        let max_offset = (total_lines + padding_bottom).saturating_sub(height);

        let available = height.saturating_sub(1);
        let margin_top = margin_top.min(available);
        let margin_bottom = margin_bottom.min(available - margin_top);

        let top_boundary = current_scroll + margin_top;
        let bottom_boundary = current_scroll + height.saturating_sub(margin_bottom + 1);

        if current_index < top_boundary {
            // Scroll up
            current_scroll.saturating_sub(top_boundary - current_index)
        } else if current_index > bottom_boundary {
            // Scroll down
            let delta = current_index - bottom_boundary;
            (current_scroll + delta).min(max_offset)
        } else {
            // No scroll
            current_scroll
        }
    }

    pub fn is_scrollable(total_lines: usize, area: &mut Rect) -> Option<Rect> {
        Self::is_scrollable_with_options(total_lines, area, 10, 2)
    }

    pub fn is_scrollable_with_options(
        total_lines: usize,
        area: &mut Rect,
        min_width: u16,
        scrollbar_gap: u16,
    ) -> Option<Rect> {
        let scrollable = total_lines > area.height as usize && area.width > min_width;
        scrollable.then(|| {
            let scroll_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                width: 1,
                ..*area
            };
            area.width = area.width.saturating_sub(1 + scrollbar_gap);
            scroll_area
        })
    }
}

pub struct ScrollbarColors {
    pub thumb: Color,
    pub track: Option<Color>,
}

impl ScrollbarColors {
    pub const fn new(thumb: Color, track: Option<Color>) -> Self {
        Self { thumb, track }
    }
}

impl Default for ScrollbarColors {
    fn default() -> Self {
        Self::new(Color::DarkGray, None)
    }
}

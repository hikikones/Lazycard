use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style},
};

pub struct Scrollbar {
    colors: ScrollbarColors,
}

impl Scrollbar {
    pub const fn new() -> Self {
        Self {
            colors: ScrollbarColors::DEFAULT,
        }
    }

    pub const fn with_colors(mut self, colors: ScrollbarColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn render(
        self,
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

    pub const fn calculate_scroll(
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

    pub const fn calculate_scroll_with_margins(
        total_lines: usize,
        viewport_height: u16,
        current_index: usize,
        current_scroll: usize,
        margin_top: usize,
        margin_bottom: usize,
        padding_bottom: usize,
    ) -> usize {
        const fn min(a: usize, b: usize) -> usize {
            if a < b { a } else { b }
        }

        let height = viewport_height as usize;
        let max_offset = (total_lines + padding_bottom).saturating_sub(height);

        let available = height.saturating_sub(1);
        let margin_top = min(margin_top, available);
        let margin_bottom = min(margin_bottom, available - margin_top);

        let top_boundary = current_scroll + margin_top;
        let bottom_boundary = current_scroll + height.saturating_sub(margin_bottom + 1);

        if current_index < top_boundary {
            // Scroll up
            current_scroll.saturating_sub(top_boundary - current_index)
        } else if current_index > bottom_boundary {
            // Scroll down
            let delta = current_index - bottom_boundary;
            min(current_scroll + delta, max_offset)
        } else {
            // No scroll
            current_scroll
        }
    }

    pub const fn is_scrollable(total_lines: usize, viewport_size: Size) -> bool {
        Self::is_scrollable_with_options(total_lines, viewport_size, 15)
    }

    pub const fn is_scrollable_with_options(
        total_lines: usize,
        viewport_size: Size,
        min_width: u16,
    ) -> bool {
        total_lines > viewport_size.height as usize && viewport_size.width > min_width
    }

    pub const fn make_scroll_area(area: &mut Rect) -> Rect {
        Self::make_scroll_area_with_margin(area, 1)
    }

    pub const fn make_scroll_area_with_margin(area: &mut Rect, margin: u16) -> Rect {
        let scroll_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            width: 1,
            ..*area
        };
        area.width = area.width.saturating_sub(1 + margin);
        scroll_area
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarColors {
    pub thumb: Color,
    pub track: Option<Color>,
}

impl ScrollbarColors {
    pub const DEFAULT: Self = Self {
        thumb: Color::DarkGray,
        track: None,
    };

    pub const fn new(thumb: Color, track: Option<Color>) -> Self {
        Self { thumb, track }
    }
}

impl Default for ScrollbarColors {
    fn default() -> Self {
        Self::DEFAULT
    }
}

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Rect, Size},
};

pub struct TokenList {
    index: usize,
    index_col: u16,
    index_row: u16,
    scroll: u16,
    total_lines: u16,
    total_items: usize,
    size: Size,
    gap: u16,
}

pub trait TokenItem {
    fn width(&self) -> u16;
}

impl TokenList {
    pub const fn new() -> Self {
        Self {
            index: 0,
            index_col: 0,
            index_row: 0,
            scroll: 0,
            total_lines: 0,
            total_items: 0,
            size: Size::ZERO,
            gap: 2,
        }
    }

    pub const fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub const fn index(&self) -> usize {
        self.index
    }

    pub const fn set_index(&mut self, i: usize) -> &mut Self {
        self.index = i;
        self
    }

    pub fn input<T: TokenItem>(
        &mut self,
        key: KeyCode,
        items: impl IntoIterator<Item = T>,
    ) -> bool {
        let old_index = self.index;

        match key {
            KeyCode::Right => {
                self.index = (self.index + 1).min(self.total_items.saturating_sub(1));
            }
            KeyCode::Left => {
                self.index = self.index.saturating_sub(1);
            }
            KeyCode::Down => {
                self.index = if self.index_row == self.total_lines.saturating_sub(1) {
                    self.total_items.saturating_sub(1)
                } else {
                    let (mut next_index, mut distance) = (0, u16::MAX);
                    for (i, x, y, _) in
                        iter_items(self.size.width, self.gap, items).skip(self.index + 1)
                    {
                        if y == self.index_row + 1 {
                            let d = self.index_col.abs_diff(x);
                            if d <= distance {
                                next_index = i;
                                distance = d;
                            }
                        } else if y > self.index_row + 1 {
                            break;
                        }
                    }
                    next_index
                };
            }
            KeyCode::Up => {
                self.index = if self.index_row == 0 {
                    0
                } else {
                    let (mut next_index, mut distance) = (0, u16::MAX);
                    for (i, x, y, _) in iter_items(self.size.width, self.gap, items) {
                        if y == self.index_row.saturating_sub(1) {
                            let d = self.index_col.abs_diff(x);
                            if d <= distance {
                                next_index = i;
                                distance = d;
                            }
                        } else if y >= self.index_row {
                            break;
                        }
                    }
                    next_index
                };
            }
            KeyCode::Home => {
                self.index = 0;
            }
            KeyCode::End => {
                self.index = self.total_items.saturating_sub(1);
            }
            _ => {}
        }

        self.index != old_index
    }

    pub fn render<T: TokenItem>(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        items: impl IntoIterator<Item = T, IntoIter: Clone>,
        mut render_item: impl FnMut(Rect, &mut Buffer, T, bool),
    ) {
        let items = items.into_iter();

        // Process all items every render for index and scroll data
        self.total_items = 0;
        for (i, x, y, _) in iter_items(area.width, self.gap, items.clone()) {
            if self.index == i {
                self.index_col = x;
                self.index_row = y;
            }
            self.total_items += 1;
            self.total_lines = y + 1;
        }

        // Determine scroll
        let scroll = if self.size.height != area.height {
            // Refresh scroll on window resize
            0
        } else {
            self.scroll
        };
        self.scroll = crate::calculate_scroll(
            self.total_lines as usize,
            area.height,
            self.index_row as usize,
            scroll as usize,
            0,
            0,
            0,
        ) as u16;

        self.size = Size::from(area);

        // Render
        for (i, x, y, item) in iter_items(area.width, self.gap, items) {
            if y >= area.height + self.scroll {
                break;
            }

            if y >= self.scroll {
                let area = Rect {
                    x: area.x + x,
                    y: area.y + y.saturating_sub(self.scroll),
                    width: item.width().min(area.width.saturating_sub(x + 1)),
                    height: 1,
                };
                render_item(area, buf, item, self.index == i);
            }
        }
    }
}

fn iter_items<T: TokenItem>(
    width: u16,
    gap: u16,
    items: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = (usize, u16, u16, T)> {
    let (mut x, mut y) = (0, 0);
    items.into_iter().enumerate().map(move |(i, item)| {
        if x + item.width() >= width {
            x = 0;
            if i > 0 {
                y += 1;
            }
        }

        let (col, row) = (x, y);

        x += item.width() + gap;

        (i, col, row, item)
    })
}

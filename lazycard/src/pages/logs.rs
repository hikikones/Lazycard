use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
};
use widgets::{List, ListItem, Shortcut, Shortcuts, TextSegment};

use crate::settings::Colors;

pub struct LogsPage {
    logs: Vec<Log>,
    queue: u32,
    list: List,
    horizontal_scroll: usize,
}

pub enum LogsAction {
    None,
    Render,
    Done,
}

impl LogsPage {
    pub const fn new() -> Self {
        Self {
            logs: Vec::new(),
            queue: 0,
            list: List::new(),
            horizontal_scroll: 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    pub fn enqueue(&mut self, log: Log) {
        self.logs.push(log);
        self.queue += 1;
    }

    pub const fn queue_len(&self) -> u32 {
        self.queue
    }

    pub fn on_enter(&mut self) {
        self.queue = 0;
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut TextSegment,
        shortcuts: &mut Shortcuts,
    ) {
        if self.logs.is_empty() {
            widgets::print_ascii(
                area,
                buf,
                "No logs to report",
                colors.neutral,
                Some(widgets::Alignment::Center),
            );
            return;
        }

        menu.push_str("Logs (", colors.neutral);
        menu.push_int(self.logs.len(), colors.neutral);
        menu.push_char(')', colors.neutral);

        // Render logs
        self.list.set_colors(colors.neutral, None).render(
            area,
            buf,
            self.logs.iter(),
            |line, buf, log, item| {
                let (scroll, style) = if item == ListItem::Selected {
                    let max_scroll = log.width.saturating_sub(line.width as usize);
                    self.horizontal_scroll = max_scroll.min(self.horizontal_scroll);
                    (
                        self.horizontal_scroll,
                        Style::new().fg(colors.primary).reversed(),
                    )
                } else {
                    (0, Style::new())
                };

                widgets::print_text(line, buf, &log.message[scroll..], style, true, None);
            },
        );

        // Shortcuts
        shortcuts.push(Shortcut::new("Clear", "c"));
    }

    pub fn on_input(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> LogsAction {
        if self.logs.is_empty() {
            return LogsAction::None;
        }

        match key {
            KeyCode::Right => {
                self.horizontal_scroll += 1;
                return LogsAction::Render;
            }
            KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                return LogsAction::Render;
            }
            KeyCode::Char('c') => {
                self.logs.clear();
                self.horizontal_scroll = 0;
                self.list.set_index(0);
                return LogsAction::Done;
            }
            _ => {
                if self.list.input(key, KeyModifiers::empty()) {
                    self.horizontal_scroll = 0;
                    return LogsAction::Render;
                }
            }
        }

        LogsAction::None
    }

    pub fn on_exit(&self) {}
}

pub struct Log {
    message: String,
    width: usize,
}

impl Log {
    pub fn new(message: impl ToString) -> Self {
        let message = message.to_string();
        let width = unicode_width::UnicodeWidthStr::width(message.as_str());
        Self { message, width }
    }
}

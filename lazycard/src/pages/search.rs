use database::{CardId, Database};
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Block, Padding},
};
use widgets::{
    KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextInput, TextInputColors, TextSegment,
};

use crate::{app::AppInput, settings::Colors, symbols};

pub struct SearchPage {
    state: State,
    search_input: TextInput,
    search_results: Vec<CardId>,
    index: usize,
    is_empty: bool,
}

enum State {
    Search,
    Browse,
}

pub enum SearchAction {
    None,
    Render,
    Goto(Option<CardId>),
    Done,
}

impl SearchPage {
    pub const fn new() -> Self {
        Self {
            state: State::Search,
            search_input: TextInput::new().with_placeholder("Search..."),
            search_results: Vec::new(),
            index: 0,
            is_empty: false,
        }
    }

    pub fn on_enter(&mut self, db: &Database, markup: &mut Markup) {
        self.state = State::Search;
        self.index = 0;
        self.is_empty = db.is_empty().unwrap();
        markup.set_max_items(None);
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        db: &Database,
        menu: &mut TextSegment,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
        colors: &Colors,
    ) {
        menu.push_str("Search", colors.neutral);

        if self.is_empty {
            widgets::print_ascii(
                area,
                buf,
                "No cards to search for",
                colors.neutral,
                Some(widgets::Alignment::Center),
            );
            return;
        }

        // Determine colors and shortcuts for search input and results
        let (border_style, border_text_color) = {
            match self.state {
                State::Search => {
                    self.search_input.set_colors(TextInputColors {
                        normal: Color::Reset,
                        cursor: colors.primary,
                        selector: colors.neutral,
                        placeholder: colors.neutral,
                        disabled: colors.neutral,
                    });
                    shortcuts.push(Shortcut::new("Browse", symbols::ENTER));
                    (Style::new().fg(colors.neutral), colors.neutral)
                }
                State::Browse => {
                    self.search_input
                        .set_colors(TextInputColors::all(colors.neutral));
                    shortcuts.extend([Shortcut::new("Search", "s")]);
                    (Style::new().fg(colors.secondary), Color::Reset)
                }
            }
        };

        // Render input
        let search_line =
            Rect { height: 1, ..area }.centered_horizontally(Constraint::Percentage(64));
        self.search_input.render(search_line, buf);

        // Render results
        let results_area = Rect {
            y: area.y + search_line.height + 1,
            height: area.height.saturating_sub(search_line.height + 1),
            ..area
        };
        let results_block = Block::bordered()
            .border_style(border_style)
            .padding(Padding::horizontal(1));
        let results_inner = results_block.inner(results_area);
        results_block.render(results_area, buf);

        // Title for bordered search results
        utils::format_int(self.search_results.len(), |len| {
            widgets::print_asciis(
                Rect {
                    y: results_area.y,
                    height: 1,
                    ..results_inner
                },
                buf,
                [" Search Results (", len, ") "],
                border_text_color,
                Some(widgets::Alignment::CenterHorizontal),
            );
        });

        match self.search_results.get(self.index).copied() {
            Some(id) => {
                db.get_card_content(id, |content| {
                    markup.render(results_inner, buf, content, kitty);
                })
                .unwrap();
            }
            None => {
                //todo?
            }
        }
    }

    pub fn on_input(
        &mut self,
        input: AppInput,
        db: &Database,
        markup: &mut Markup,
    ) -> SearchAction {
        if self.is_empty {
            return SearchAction::None;
        }

        let (key, modifiers) = input.key_pressed_and_modifiers();

        match self.state {
            State::Search => match key {
                KeyCode::Enter => {
                    self.search_results.clear();
                    let _ = db.search(self.search_input.as_str_trim(), &mut self.search_results);
                    if !self.search_results.is_empty() {
                        self.state = State::Browse;
                    }
                    return SearchAction::Render;
                }
                KeyCode::Down => {
                    if !self.search_results.is_empty() {
                        self.state = State::Browse;
                        return SearchAction::Render;
                    }
                }
                KeyCode::Up => {}
                _ => {
                    if self.search_input.input(key, modifiers) {
                        return SearchAction::Render;
                    }
                }
            },
            State::Browse => match key {
                KeyCode::Right => {
                    if self.search_results.len() > 1 {
                        self.index = (self.index + 1) % self.search_results.len();
                        markup.scroll(ScrollMove::Start);
                        return SearchAction::Render;
                    }
                }
                KeyCode::Left => {
                    if self.search_results.len() > 1 {
                        if self.index == 0 {
                            self.index = self.search_results.len() - 1;
                        } else {
                            self.index -= 1;
                        }
                        markup.scroll(ScrollMove::Start);
                        return SearchAction::Render;
                    }
                }
                KeyCode::Char('s') => {
                    self.state = State::Search;
                    return SearchAction::Render;
                }
                _ => {
                    if markup.input(key) {
                        return SearchAction::Render;
                    }
                }
            },
        }

        SearchAction::None
    }

    pub fn on_exit(&mut self) {
        self.search_input.clear();
        self.search_results.clear();
    }
}

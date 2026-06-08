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
    search: TextInput,
    results: Vec<CardId>,
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
    Edit(Option<CardId>),
    Goto(Option<CardId>),
}

impl SearchPage {
    pub const fn new(colors: &Colors) -> Self {
        Self {
            state: State::Search,
            search: TextInput::new()
                .with_placeholder("Search...")
                .with_colors(TextInputColors {
                    normal: Color::Reset,
                    cursor: colors.primary,
                    selector: colors.neutral,
                    placeholder: colors.neutral,
                    disabled: colors.neutral,
                }),
            results: Vec::new(),
            index: 0,
            is_empty: false,
        }
    }

    pub fn on_enter(&mut self, db: &Database, markup: &mut Markup) {
        self.state = State::Search;
        self.is_empty = db.is_empty().unwrap();
        self.refresh(db);
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
        let (border_color, border_text_color) = {
            match self.state {
                State::Search => {
                    shortcuts.push(Shortcut::new("Confirm", symbols::ENTER));
                    (colors.neutral, colors.neutral)
                }
                State::Browse => {
                    if self.current_card().is_some() {
                        shortcuts.extend([Shortcut::new("Edit", "e"), Shortcut::new("Goto", "g")]);
                    }
                    shortcuts.push(Shortcut::new("Search", "s"));
                    (colors.secondary, Color::Reset)
                }
            }
        };

        // Render input
        let search_line =
            Rect { height: 1, ..area }.centered_horizontally(Constraint::Percentage(64));
        self.search
            .set_enabled(matches!(self.state, State::Search))
            .render(search_line, buf);

        // Render results
        let results_area = Rect {
            y: area.y + search_line.height + 1,
            height: area.height.saturating_sub(search_line.height + 1),
            ..area
        };
        let results_block = Block::bordered()
            .border_style(border_color)
            .padding(Padding::horizontal(1));
        let card_area = results_block.inner(results_area);
        results_block.render(results_area, buf);

        // Title for bordered search results
        utils::format_int2(self.index + 1, self.results.len(), |i, len| {
            widgets::print_asciis(
                Rect {
                    y: results_area.y,
                    height: 1,
                    ..card_area
                },
                buf,
                [" ", i, " / ", len, " "],
                border_text_color,
                Some(widgets::Alignment::CenterHorizontal),
            );
        });

        match self.current_card() {
            Some(id) => {
                db.get_card_content(id, |content| {
                    markup.render(card_area, buf, content, kitty);
                })
                .unwrap();
            }
            None => {
                if !self.search.is_empty_trim() {
                    widgets::print_ascii(
                        area,
                        buf,
                        "No card found",
                        colors.neutral,
                        Some(widgets::Alignment::Center),
                    );
                }
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
                    let input = self.search.as_str_trim();
                    if !input.is_empty() {
                        self.results.clear();
                        self.index = 0;
                        let _ = db.search(input, &mut self.results);
                        if !self.results.is_empty() {
                            self.state = State::Browse;
                        }
                        return SearchAction::Render;
                    }
                }
                KeyCode::Down => {
                    if !self.results.is_empty() {
                        self.state = State::Browse;
                        return SearchAction::Render;
                    }
                }
                KeyCode::Up => {}
                _ => {
                    if self.search.input(key, modifiers) {
                        return SearchAction::Render;
                    }
                }
            },
            State::Browse => match key {
                KeyCode::Up => {
                    if markup.scroll_index() == 0 {
                        self.state = State::Search;
                        return SearchAction::Render;
                    } else if markup.scroll(ScrollMove::Up) {
                        return SearchAction::Render;
                    }
                }
                KeyCode::Right => {
                    if self.results.len() > 1 {
                        self.index = (self.index + 1) % self.results.len();
                        markup.scroll(ScrollMove::Start);
                        return SearchAction::Render;
                    }
                }
                KeyCode::Left => {
                    if self.results.len() > 1 {
                        if self.index == 0 {
                            self.index = self.results.len() - 1;
                        } else {
                            self.index -= 1;
                        }
                        markup.scroll(ScrollMove::Start);
                        return SearchAction::Render;
                    }
                }
                KeyCode::Char('e') => {
                    return SearchAction::Edit(self.current_card());
                }
                KeyCode::Char('g') => {
                    return SearchAction::Goto(self.current_card());
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
        // self.state = State::Search;
        // self.search.clear();
        // self.results.clear();
        // self.index = 0;
        //todo?
    }

    fn current_card(&self) -> Option<CardId> {
        self.results.get(self.index).copied()
    }

    fn refresh(&mut self, db: &Database) {
        let input = self.search.as_str_trim();
        if self.is_empty || input.is_empty() {
            self.search.clear();
            self.results.clear();
            self.index = 0;
            return;
        }

        self.results.clear();
        let _ = db.search(input, &mut self.results);
        self.index = self.index.min(self.results.len().saturating_sub(1));
    }
}

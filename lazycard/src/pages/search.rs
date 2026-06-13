use database::{CardId, Database};
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Block, Padding},
};
use widgets::{KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextInput, TextSegment};

use crate::{app::AppInput, settings::Colors, symbols};

// TODO: Render a help text as markup when search comes up empty.
// Or just add a help shortcut that shows how to search.

pub struct SearchPage {
    state: State,
    search: TextInput,
    results: Vec<CardId>,
    index: usize,
    query: String,
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
                .with_colors(colors.text_input()),
            results: Vec::new(),
            index: 0,
            query: String::new(),
            is_empty: false,
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.state = State::Search;
        self.is_empty = db.is_empty().unwrap();
        self.refresh(db);
    }

    pub fn on_render(
        &mut self,
        mut area: Rect,
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

        // Determine colors and shortcuts for search and results
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

        // Render search input
        let search_line = widgets::align(
            Rect {
                width: (0.64 * area.width as f32).round() as u16,
                height: 1,
                ..area
            },
            area,
            widgets::Alignment::CenterHorizontal,
        );
        self.search
            .set_enabled(matches!(self.state, State::Search))
            .render(search_line, buf);

        area.y += 2;
        area.height = area.height.saturating_sub(2);

        // Results block
        let card_block = Block::bordered()
            .border_style(border_color)
            .padding(Padding::horizontal(1));
        let card_area = card_block.inner(area);
        card_block.render(area, buf);

        // Title for block
        utils::format_int2(self.index + 1, self.results.len(), |i, len| {
            widgets::print_asciis(
                area,
                buf,
                [" ", i, " / ", len, " "],
                border_text_color,
                Some(widgets::Alignment::CenterHorizontal),
            );
        });

        // Render search results
        match self.current_card() {
            Some(id) => {
                db.get_card_content(id, |content| {
                    markup.render(card_area, buf, content, kitty);
                })
                .unwrap();
            }
            None => {
                if !self.query.is_empty() {
                    if self.query.chars().count() < 3 {
                        widgets::print_ascii(
                            card_area,
                            buf,
                            "Search query must be at least 3 characters",
                            colors.neutral,
                            Some(widgets::Alignment::Center),
                        );
                    } else {
                        let center = widgets::align(
                            Rect {
                                height: 2,
                                ..card_area
                            },
                            card_area,
                            widgets::Alignment::Center,
                        );
                        widgets::print_ascii(
                            center,
                            buf,
                            "No cards found from query",
                            colors.neutral,
                            Some(widgets::Alignment::CenterHorizontal),
                        );
                        widgets::print_ascii(
                            Rect {
                                y: center.y + 1,
                                ..center
                            },
                            buf,
                            self.query.as_str(),
                            Style::new().fg(colors.neutral).italic(),
                            Some(widgets::Alignment::CenterHorizontal),
                        );
                    }
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
                        self.query.clear();
                        self.query.push_str(input);
                        let _ = db.search(input, |id| self.results.push(id));
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
        if self.is_empty || self.query.is_empty() {
            self.results.clear();
            self.index = 0;
            return;
        }

        self.results.clear();
        let _ = db.search(self.query.as_str(), |id| self.results.push(id));
        self.index = self.index.min(self.results.len().saturating_sub(1));
    }
}

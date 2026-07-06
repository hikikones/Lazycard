use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Block, Padding},
};
use widgets::{KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextInput};

use crate::{
    app::{Action, AppInput, AppRender},
    database::{CardId, Database},
    pages::{CardsParam, Route},
    settings::Colors,
    symbols,
};

// TODO: Render a help text as markup when search comes up empty.
// Or just add a help shortcut that shows how to search.

pub struct SearchPage {
    state: State,
    search: TextInput,
    results: Vec<CardId>,
    index: usize,
    query: String,
    content: String,
    is_empty: bool,
}

enum State {
    Search,
    Browse,
}

impl SearchPage {
    pub const fn new() -> Self {
        Self {
            state: State::Search,
            search: TextInput::new().with_placeholder("Search..."),
            results: Vec::new(),
            index: 0,
            query: String::new(),
            content: String::new(),
            is_empty: false,
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.state = State::Search;
        self.is_empty = db.is_cards_empty();
        self.refresh(db);
    }

    pub fn on_render(
        &mut self,
        render: AppRender,
        colors: &Colors,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        let (mut area, buf) = render.area_and_buffer();

        widgets::print_ascii(
            area,
            buf,
            "Search",
            colors.neutral,
            Some(widgets::Alignment::CenterHorizontal),
        );

        area.height = area.height.saturating_sub(2);
        area.y += 2;

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
            .set_colors(colors.text_input())
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
            Some(_id) => {
                markup.render(card_area, buf, self.content.as_str(), kitty);
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

    pub fn on_input(&mut self, input: AppInput, db: &Database, markup: &mut Markup) -> Action {
        if self.is_empty {
            return Action::None;
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
                        if let Some(id) = self.current_card() {
                            self.highlight(id, db);
                            self.state = State::Browse;
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Down => {
                    if !self.results.is_empty() {
                        self.state = State::Browse;
                        return Action::Render;
                    }
                }
                KeyCode::Up => {}
                _ => {
                    if self.search.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            },
            State::Browse => match key {
                KeyCode::Up => {
                    if markup.scroll_index() == 0 {
                        self.state = State::Search;
                        return Action::Render;
                    } else if markup.scroll(ScrollMove::Up) {
                        return Action::Render;
                    }
                }
                KeyCode::Right => {
                    if self.results.len() > 1 {
                        self.index = (self.index + 1) % self.results.len();
                        self.highlight(self.current_card().unwrap(), db);
                        markup.scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                KeyCode::Left => {
                    if self.results.len() > 1 {
                        if self.index == 0 {
                            self.index = self.results.len() - 1;
                        } else {
                            self.index -= 1;
                        }
                        self.highlight(self.current_card().unwrap(), db);
                        markup.scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(id) = self.current_card() {
                        return Action::Route(Route::Editor(Some(id)));
                    }
                }
                KeyCode::Char('g') => {
                    if let Some(id) = self.current_card() {
                        return Action::Route(Route::Cards(Some(CardsParam::Card(id))));
                    }
                }
                KeyCode::Char('s') => {
                    self.state = State::Search;
                    return Action::Render;
                }
                _ => {
                    if markup.input(key) {
                        return Action::Render;
                    }
                }
            },
        }

        Action::None
    }

    pub fn on_exit(&self) {}

    fn current_card(&self) -> Option<CardId> {
        self.results.get(self.index).copied()
    }

    fn highlight(&mut self, id: CardId, db: &Database) {
        self.content.clear();
        db.search_highlight(id, &self.query, |content| {
            self.content.push_str(content);
        });
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

        if let Some(id) = self.current_card() {
            self.highlight(id, db);
        }
    }
}

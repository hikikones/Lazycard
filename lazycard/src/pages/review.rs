use database::{CardId, Database};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::Style};
use widgets::{KittyGraphics, Markup, MarkupItem, ScrollMove, Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput},
    pages::Route,
    settings::Colors,
    symbols,
};

pub struct ReviewPage {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    state: ReviewState,
    markup_items: Vec<MarkupItem>,
    reveal_len: usize,
    rng: fastrand::Rng,
}

enum ReviewState {
    None,
    Review(CardId),
    Done,
}

impl ReviewPage {
    pub fn new() -> Self {
        Self {
            due: Vec::new(),
            total: 0,
            progress: 0,
            state: ReviewState::None,
            markup_items: Vec::new(),
            reveal_len: 0,
            rng: fastrand::Rng::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database, markup: &mut Markup) {
        db.get_due_cards(&mut self.due).unwrap();
        self.total = self.due.len();

        if self.total > 0 {
            self.next_card(db, markup);
        }
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        db: &Database,
        menu: &mut TextSegment,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        match self.state {
            ReviewState::None => {
                widgets::print_ascii(
                    area,
                    buf,
                    "No cards to review",
                    Style::new(),
                    Some(widgets::Alignment::Center),
                );
            }
            ReviewState::Review(id) => {
                menu.push_int(self.progress, colors.neutral);
                menu.push_str(" / ", colors.neutral);
                menu.push_int(self.total, colors.neutral);

                db.get_card_content(id, |content| {
                    markup
                        .set_max_items(Some(self.reveal_len))
                        .render(area, buf, content, kitty);
                })
                .unwrap();

                if self.is_fully_revealed() {
                    shortcuts.extend([Shortcut::new("Yes", "y"), Shortcut::new("No", "n")]);
                } else {
                    shortcuts.extend([Shortcut::new("Show", symbols::SPACE)]);
                }

                if !self.due.is_empty() {
                    shortcuts.push(Shortcut::new("Skip", symbols::ARROW_RIGHT));
                }

                shortcuts.extend([
                    Shortcut::new("Edit", "e"),
                    Shortcut::new("Delete", symbols::DELETE),
                ]);
            }
            ReviewState::Done => {
                widgets::print_ascii(
                    area,
                    buf,
                    "Good job!",
                    Style::new(),
                    Some(widgets::Alignment::Center),
                );
            }
        }
    }

    pub fn on_input(&mut self, input: AppInput, markup: &mut Markup, db: &mut Database) -> Action {
        let key = input.key_pressed();
        match self.state {
            ReviewState::Review(id) => match key {
                KeyCode::Char('e') => return Action::Route(Route::Editor(Some(id))),
                KeyCode::Delete => {
                    db.delete_card(id).unwrap();
                    self.total = self.total.saturating_sub(1);
                    self.next_card(db, markup);
                    return Action::Render;
                }
                KeyCode::Char(' ') => {
                    if !self.is_fully_revealed() {
                        self.reveal_more();
                        markup.set_desired_scroll(ScrollMove::End);
                        return Action::Render;
                    }
                }
                KeyCode::Char('y' | 'n') => {
                    if self.is_fully_revealed() {
                        let success = key == KeyCode::Char('y');
                        db.review_card(id, success).unwrap();
                        self.progress += 1;
                        self.next_card(db, markup);
                        return Action::Render;
                    }
                }
                KeyCode::Right => {
                    if !self.due.is_empty() {
                        self.next_card(db, markup);
                        self.due.push(id);
                        return Action::Render;
                    }
                }
                _ => {
                    if markup.input(key) {
                        return Action::Render;
                    }
                }
            },
            ReviewState::None | ReviewState::Done => {}
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.total = 0;
        self.progress = 0;
        self.state = ReviewState::None;
        self.markup_items.clear();
        self.reveal_len = 0;
    }

    fn next_card(&mut self, db: &Database, markup: &mut Markup) {
        if self.due.is_empty() {
            self.state = ReviewState::Done;
            return;
        }

        let random_index = self.rng.usize(0..self.due.len());
        let id = self.due.swap_remove(random_index);

        db.get_card_content(id, |content| {
            self.markup_items.clear();
            Markup::parse_items(content, &mut self.markup_items);
        })
        .unwrap();

        self.reveal_len = 0;
        self.state = ReviewState::Review(id);
        self.reveal_more();
        markup.scroll(ScrollMove::Start);
    }

    fn reveal_more(&mut self) {
        self.reveal_len = self
            .markup_items
            .iter()
            .copied()
            .enumerate()
            .filter(|&(i, b)| matches!(b, MarkupItem::Break) && i > self.reveal_len)
            .map(|(i, _)| i)
            .next()
            .unwrap_or(self.markup_items.len());
    }

    const fn is_fully_revealed(&self) -> bool {
        self.reveal_len == self.markup_items.len()
    }
}

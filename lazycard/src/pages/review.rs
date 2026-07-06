use ratatui::{crossterm::event::KeyCode, style::Style};
use widgets::{KittyGraphics, Markup, MarkupItem, ScrollMove, Shortcut, Shortcuts};

use crate::{
    app::{Action, AppInput, AppRender},
    database::{CardId, Database},
    pages::Route,
    settings::Colors,
    symbols,
};

pub struct ReviewPage {
    total: u32,
    progress: u32,
    state: ReviewState,
    markup_items: Vec<MarkupItem>,
    reveal_len: usize,
    desired_retention: f32,
}

enum ReviewState {
    None,
    Review(CardId),
    Done,
}

impl ReviewPage {
    pub const fn new() -> Self {
        Self {
            total: 0,
            progress: 0,
            state: ReviewState::None,
            markup_items: Vec::new(),
            reveal_len: 0,
            desired_retention: 0.0,
        }
    }

    pub const fn set_desired_retention(&mut self, retention: f32) {
        self.desired_retention = retention;
    }

    pub fn on_enter(&mut self, db: &Database, markup: &mut Markup) {
        self.total = db.get_due_count();

        if self.total > 0 {
            self.next_card(db, markup);
        }
    }

    pub fn on_render(
        &mut self,
        render: AppRender,
        db: &Database,
        colors: &Colors,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        let (mut area, buf) = render.area_and_buffer();

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
                utils::format_int2(self.progress + 1, self.total, |progress, total| {
                    widgets::print_asciis(
                        area,
                        buf,
                        [progress, " / ", total],
                        colors.neutral,
                        Some(widgets::Alignment::CenterHorizontal),
                    );
                });

                area.height = area.height.saturating_sub(2);
                area.y += 2;

                db.get_card_content(id, |content| {
                    markup
                        .set_max_items(Some(self.reveal_len))
                        .render(area, buf, content, kitty);
                });

                if self.is_fully_revealed() {
                    shortcuts.extend([Shortcut::new("Yes", "y"), Shortcut::new("No", "n")]);
                } else {
                    shortcuts.extend([Shortcut::new("Show", symbols::SPACE)]);
                }

                if self.has_more_cards() {
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
                    db.delete_card(id);
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
                        db.review_card(id, success, self.desired_retention);
                        self.progress += 1;
                        if self.is_done() {
                            self.state = ReviewState::Done;
                        } else {
                            self.next_card(db, markup);
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Right => {
                    if self.has_more_cards() {
                        self.next_card(db, markup);
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
        self.total = 0;
        self.progress = 0;
        self.state = ReviewState::None;
        self.markup_items.clear();
        self.reveal_len = 0;
    }

    fn next_card(&mut self, db: &Database, markup: &mut Markup) {
        let next_card = if let ReviewState::Review(id) = self.state {
            db.get_due_card_random_except(id)
        } else {
            db.get_due_card_random()
        };

        match next_card {
            Some(id) => {
                db.get_card_content(id, |content| {
                    self.markup_items.clear();
                    Markup::parse_items(content, &mut self.markup_items);
                });
                self.reveal_len = 0;
                self.state = ReviewState::Review(id);
                self.reveal_more();
                markup.scroll(ScrollMove::Start);
            }
            None => {
                self.state = ReviewState::Done;
            }
        }
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

    const fn is_done(&self) -> bool {
        self.progress == self.total
    }

    const fn has_more_cards(&self) -> bool {
        (self.total - self.progress) > 1
    }
}

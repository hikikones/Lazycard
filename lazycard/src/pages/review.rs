use database::{CardId, Database};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::Style,
    text::{Line, Span},
};
use widgets::{BreakParser, Markup, ScrollMove, Shortcut, ShortcutLine, Shortcuts};

use crate::{
    app::{Action, CardsIterExt},
    pages::Route,
    settings::Colors,
};

pub struct ReviewPage {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    state: ReviewState,
    reveals: Vec<String>,
    text: String,
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
            reveals: Vec::new(),
            text: String::new(),
            rng: fastrand::Rng::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().due().map(|(id, _)| id));
        self.total = self.due.len();

        if let Some(id) = self.next_card() {
            self.start_review(id, db);
        }
    }

    fn next_card(&mut self) -> Option<CardId> {
        if self.due.is_empty() {
            None
        } else {
            let random_index = self.rng.usize(0..self.due.len());
            Some(self.due.remove(random_index))
        }
    }

    fn start_review(&mut self, id: CardId, db: &Database) {
        self.reveals.clear();
        self.text.clear();

        let card_content = db.get(id).unwrap().content.as_str();

        let mut start = 0;
        for i in BreakParser::new(card_content) {
            self.reveals.push(card_content[start..i].to_owned());
            start = i;
        }
        self.reveals.push(card_content[start..].to_owned());
        self.reveals.reverse();
        self.state = ReviewState::Review(id);
        self.reveal_next();
    }

    fn reveal_next(&mut self) -> bool {
        if let Some(s) = self.reveals.pop() {
            self.text.push_str(s.as_str());
            true
        } else {
            false
        }
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut Line,
        markup: &mut Markup,
        shortcuts: &mut Shortcuts,
    ) {
        match self.state {
            ReviewState::None => {
                markup.render("| No cards to review", area, buf);
            }
            ReviewState::Review(_) => {
                menu.push_span(Span::styled(
                    format!("{} / {}", self.progress, self.total),
                    Style::new().fg(colors.neutral),
                ));

                markup.render(&self.text, area, buf);

                if !self.reveals.is_empty() {
                    shortcuts.extend(ShortcutLine::Top, [Shortcut::new("Show", "Space")]);
                } else {
                    shortcuts.extend(
                        ShortcutLine::Top,
                        [Shortcut::new("Yes", "y"), Shortcut::new("No", "n")],
                    );
                }
                if !self.due.is_empty() {
                    shortcuts.push(ShortcutLine::Middle, Shortcut::new("Skip", "➝"));
                }
                shortcuts.extend(
                    ShortcutLine::Middle,
                    [Shortcut::new("Edit", "e"), Shortcut::new("Archive", "Del")],
                );
            }
            ReviewState::Done => {
                markup.render("| Good job!", area, buf);
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        match self.state {
            ReviewState::Review(id) => match key {
                KeyCode::Char('e') => return Action::Route(Route::Editor(Some(id))),
                KeyCode::Delete => {
                    db.update(id, |card| card.archived = true);
                    self.total = self.total.saturating_sub(1);
                    if let Some(next_id) = self.next_card() {
                        self.start_review(next_id, db);
                        markup.desired_scroll(ScrollMove::Start);
                    } else {
                        self.state = ReviewState::Done;
                    }
                    return Action::Render;
                }
                KeyCode::Char(' ') => {
                    if self.reveal_next() {
                        markup.desired_scroll(ScrollMove::End);
                        return Action::Render;
                    }
                }
                KeyCode::Char('y' | 'n') => {
                    if self.reveals.is_empty() {
                        let success = key == KeyCode::Char('y');
                        db.schedule(id, success);
                        self.progress += 1;
                        if let Some(next_id) = self.next_card() {
                            self.start_review(next_id, db);
                            markup.desired_scroll(ScrollMove::Start);
                        } else {
                            self.state = ReviewState::Done;
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Right => {
                    if let Some(next_id) = self.next_card() {
                        self.due.push(id);
                        self.start_review(next_id, db);
                        markup.desired_scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                _ => {
                    if markup.input(key, modifiers) {
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
        self.reveals.clear();
        self.text.clear();
    }
}

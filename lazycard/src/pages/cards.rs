use database::{CardId, Database};
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, layout::Rect, style::Style};
use widgets::{KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput, Matcher},
    pages::Route,
    settings::Colors,
    symbols,
};

pub struct CardsPage {
    cards: Vec<CardId>,
    index: usize,
}

impl CardsPage {
    pub const fn new(colors: &Colors) -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        db.get_cards(&mut self.cards).unwrap();
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        db: &Database,
        colors: &Colors,
        menu: &mut TextSegment,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        match self.cards.get(self.index).copied() {
            Some(id) => {
                let neutral = Style::new().fg(colors.neutral);
                menu.push_int(self.index + 1, neutral);
                menu.push_str(" / ", neutral);
                menu.push_int(self.cards.len(), neutral);

                db.get_card_content(id, |content| {
                    markup.render(area, buf, content, kitty);
                })
                .unwrap();

                shortcuts.extend([
                    Shortcut::new("Edit", "e"),
                    Shortcut::new("Delete", symbols::DELETE),
                ]);
            }
            None => {
                widgets::print_ascii(
                    area,
                    buf,
                    "You have no cards",
                    Style::new(),
                    Some(widgets::Alignment::Center),
                );
            }
        }
    }

    pub fn on_input(
        &mut self,
        input: AppInput,
        markup: &mut Markup,
        db: &mut Database,
        matcher: &mut Matcher,
    ) -> Action {
        if self.cards.is_empty() {
            return Action::None;
        }

        let (key, modifiers) = input.key_pressed_and_modifiers();

        match key {
            KeyCode::Right => {
                if self.cards.len() > 1 {
                    self.index = (self.index + 1) % self.cards.len();
                    markup.scroll(ScrollMove::Start);
                    return Action::Render;
                }
            }
            KeyCode::Left => {
                if self.cards.len() > 1 {
                    if self.index == 0 {
                        self.index = self.cards.len() - 1;
                    } else {
                        self.index -= 1;
                    }
                    markup.scroll(ScrollMove::Start);
                    return Action::Render;
                }
            }
            KeyCode::Delete => {
                let id = self.cards.remove(self.index);
                db.delete_card(id).unwrap();

                if !self.cards.is_empty() {
                    self.index = self.index.min(self.cards.len() - 1);
                    markup.scroll(ScrollMove::Start);
                }

                return Action::Render;
            }
            KeyCode::Char('e') => {
                let id = self.cards.get(self.index).copied().unwrap();
                return Action::Route(Route::Editor(Some(id)));
            }
            _ => {
                if markup.input(key) {
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.cards.clear();
        self.index = 0;
    }
}

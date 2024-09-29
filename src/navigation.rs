use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};

use crate::database::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard,
}

pub struct Pages {
    pub review: Review,
    pub add_card: AddCard,
    pub edit_card: EditCard,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: Review::new(),
            add_card: AddCard::new(),
            edit_card: EditCard::new(),
        }
    }
}

pub struct Review {
    current: CardId,
    card: Option<Card>,
}

impl Review {
    pub const fn new() -> Self {
        Self {
            current: CardId(0),
            card: None,
        }
    }

    pub fn enter(&mut self, db: &Database) {
        if let Some((id, card)) = db.iter().filter(|(id, _)| id.0 != self.current.0).next() {
            self.current = *id;
            self.card = Some(card.clone())
        }
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        let text = match self.card.as_ref() {
            Some(card) => &card.0,
            None => "no cards to review...",
        };
        Paragraph::new(text).render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent, db: &mut Database) -> Option<Route> {
        match key.code {
            KeyCode::Null => todo!(),
            KeyCode::Right => self.enter(db),
            KeyCode::Char('e') => return Some(Route::EditCard),
            _ => {}
        }

        None
    }

    pub fn exit(&mut self) {
        self.card = None;
    }
}

pub struct AddCard;

impl AddCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn enter(&mut self, db: &Database) {
        //todo
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("add card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent, db: &mut Database) -> Option<Route> {
        match key.code {
            KeyCode::Char('s') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // todo
                }
            }
            _ => {}
        }

        None
    }

    pub fn exit(&mut self) {
        //todo
    }
}

pub struct EditCard;

impl EditCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn enter(&mut self, db: &Database) {
        //todo
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("edit card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent, db: &mut Database) -> Option<Route> {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('c') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // TODO: go back
                    return Some(Route::Review);
                }
            }
            _ => {}
        }

        None
    }

    pub fn exit(&mut self) {
        //todo
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::{InputEvent, Message},
    database::*,
};

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
    due: Vec<CardId>,
    index: usize,
    text: String,
}

impl Review {
    pub const fn new() -> Self {
        Self {
            due: Vec::new(),
            index: 0,
            text: String::new(),
        }
    }

    pub fn enter(&mut self, db: &Database) {
        self.due.extend(db.iter().map(|(id, _)| id));
        if let Some(id) = self.due.get(self.index) {
            if let Some(card) = db.get(id) {
                self.text = card.0.clone();
            }
        }
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        if self.text.is_empty() {
            Paragraph::new("no cards to review...")
                .centered()
                .render(body, buf);
            return;
        };

        Paragraph::new(self.text.as_str()).render(body, buf);
    }

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Option<Message> {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(' ') => todo!("show answer"),
                    KeyCode::Right => {
                        self.index = (self.index + 1) % self.due.len();
                        if let Some(id) = self.due.get(self.index) {
                            if let Some(card) = db.get(id) {
                                self.text = card.0.clone();
                            }
                        }
                    }
                    KeyCode::Char('e') => return Some(Message::Route(Route::EditCard)),
                    KeyCode::Tab => return Some(Message::Route(Route::AddCard)),
                    KeyCode::Esc => return Some(Message::Quit),
                    _ => {}
                }
            }
        }

        None
    }

    pub fn exit(&mut self) {
        self.due.clear();
        self.index = 0;
        self.text.clear();
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

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Option<Message> {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo
                        }
                    }
                    KeyCode::Tab => return Some(Message::Route(Route::Review)),
                    KeyCode::Esc => return Some(Message::Quit),
                    _ => {}
                }
            }
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

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Option<Message> {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo: go back
                            return Some(Message::Route(Route::Review));
                        }
                    }
                    KeyCode::Esc => return Some(Message::Quit),
                    _ => {}
                }
            }
        }

        None
    }

    pub fn exit(&mut self) {
        //todo
    }
}

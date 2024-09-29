use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::{Areas, InputEvent, Message},
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

    pub fn render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Review")
            .centered()
            .render(areas.header, buf);

        if self.text.is_empty() {
            Paragraph::new("no cards to review...")
                .centered()
                .render(areas.body, buf);
        } else {
            Paragraph::new(self.text.as_str()).render(areas.body, buf);
        }

        Line::from(
            "[esc]Quit  [tab]Menu  [e]Edit  [del]Delete  [space]Show  [↑]Yes  [↓]No  [→]Next",
        )
        .centered()
        .render(areas.footer, buf);
    }

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::AddCard),
                    KeyCode::Char('e') => return Message::Route(Route::EditCard),
                    KeyCode::Delete => todo!("delete"),
                    KeyCode::Char(' ') => todo!("show answer"),
                    KeyCode::Up => todo!("yes"),
                    KeyCode::Down => todo!("no"),
                    KeyCode::Right => {
                        self.index = (self.index + 1) % self.due.len();
                        if let Some(id) = self.due.get(self.index) {
                            if let Some(card) = db.get(id) {
                                self.text = card.0.clone();
                                return Message::Render;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Message::None
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

    pub fn render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Add Card")
            .centered()
            .render(areas.header, buf);

        Paragraph::new("add card...").render(areas.body, buf);

        Line::from("[esc]Quit  [tab]Menu  [ctrl-s]Save")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::Review),
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            todo!("save");
                        }
                    }
                    _ => {}
                }
            }
        }

        Message::None
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

    pub fn render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Edit Card")
            .centered()
            .render(areas.header, buf);

        Paragraph::new("edit card...").render(areas.body, buf);

        Line::from("[esc]Quit  [ctrl-s]Save  [ctrl-c]Cancel")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo: save and go back
                            return Message::Route(Route::Review);
                        }
                    }
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo: cancel and go back
                            return Message::Route(Route::Review);
                        }
                    }
                    _ => {}
                }
            }
        }

        Message::None
    }

    pub fn exit(&mut self) {
        //todo
    }
}

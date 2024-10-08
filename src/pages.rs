use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use ratatui_image::{picker::Picker, StatefulImage};
use tui_textarea::TextArea;

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

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().map(|(id, _)| id));
        if let Some(id) = self.due.get(self.index) {
            if let Some(card) = db.get(id) {
                self.text.push_str(card.0.as_str());
            }
        }
    }

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Review")
            .centered()
            .render(areas.header, buf);

        // if self.text.is_empty() {
        //     Paragraph::new("no cards to review...")
        //         .centered()
        //         .render(areas.body, buf);
        // } else {
        //     Paragraph::new(self.text.as_str())
        //         .wrap(Wrap { trim: false })
        //         .render(areas.body, buf);
        // }

        let dyn_img = image::ImageReader::open("glacier.jpg")
            .unwrap()
            .decode()
            .unwrap();
        let image = StatefulImage::new(None);
        // let mut picker = Picker::new((8, 12));
        let mut picker = Picker::from_termios().unwrap_or(Picker::new((8, 12)));
        picker.guess_protocol();
        let mut image2 = picker.new_resize_protocol(dyn_img);
        // image2.render(areas.body, buf);
        image.render(areas.body, buf, &mut image2);

        Line::from(
            "[esc]Quit  [tab]Menu  [e]Edit  [del]Delete  [space]Show  [↑]Yes  [↓]No  [→]Next",
        )
        .centered()
        .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::AddCard),
                    KeyCode::Char('e') => return Message::Route(Route::EditCard),
                    KeyCode::Delete => todo!("delete"),
                    KeyCode::Char(' ') => todo!("show answer"),
                    KeyCode::Up => todo!("yes"), // fixme: activates when scrolling with touchpad?
                    KeyCode::Down => todo!("no"), // fixme: activates when scrolling with touchpad?
                    KeyCode::Right => {
                        self.index = (self.index + 1) % self.due.len();
                        if let Some(id) = self.due.get(self.index) {
                            if let Some(card) = db.get(id) {
                                self.text.clear();
                                self.text.push_str(card.0.as_str());
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

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.index = 0;
        self.text.clear();
    }
}

pub struct AddCard {
    editor: TextArea<'static>,
}

impl AddCard {
    pub fn new() -> Self {
        Self {
            editor: TextArea::default(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo
    }

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Add Card")
            .centered()
            .render(areas.header, buf);

        self.editor.render(areas.body, buf);

        Line::from("[esc]Quit  [tab]Menu  [ctrl-s]Save  [ctrl-p]Preview")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::Review),
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            let content = self.editor.lines().join("\n");
                            self.editor = TextArea::default();
                            db.add(Card::new(content));
                            return Message::Render;
                        } else {
                            self.editor.input(key);
                            return Message::Render;
                        }
                    }
                    KeyCode::Char('p') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            todo!("preview");
                        } else {
                            self.editor.input(key);
                            return Message::Render;
                        }
                    }
                    _ => {
                        self.editor.input(key);
                        return Message::Render;
                    }
                }
            }
        }

        Message::None
    }

    pub fn on_exit(&mut self) {
        //todo
    }
}

pub struct EditCard;

impl EditCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo
    }

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Edit Card")
            .centered()
            .render(areas.header, buf);

        Paragraph::new("edit card...").render(areas.body, buf);

        Line::from("[esc]Quit  [ctrl-s]Save  [ctrl-c]Cancel")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
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

    pub fn on_exit(&mut self) {
        //todo
    }
}

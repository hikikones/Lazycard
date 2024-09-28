use crossterm::event::ModifierKeyCode;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard,
}

pub struct Review;

impl Review {
    pub const TITLE: &str = "Review";
    pub const FOOTER: &str = "'e' for edit";

    pub const fn new() -> Self {
        Self
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("review...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyCode) -> Option<Route> {
        if let KeyCode::Char('e') = key {
            return Some(Route::EditCard);
        }

        None
    }
}

pub struct AddCard {
    ctrl: bool,
}

impl AddCard {
    pub const TITLE: &str = "Add Card";
    pub const FOOTER: &str = "'^s' for save";

    pub const fn new() -> Self {
        Self { ctrl: false }
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("add card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyCode) -> Option<Route> {
        match key {
            KeyCode::Char('s') | KeyCode::Char('c') => {
                if self.ctrl {
                    //todo
                }
            }
            KeyCode::Modifier(ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl) => {
                self.ctrl = true;
            }
            _ => self.ctrl = false,
        }

        None
    }
}

pub struct EditCard {
    ctrl: bool,
}

impl EditCard {
    pub const TITLE: &str = "Edit Card";
    pub const FOOTER: &str = "'^s' for save, '^c' for cancel";

    pub const fn new() -> Self {
        Self { ctrl: false }
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("edit card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyCode) -> Option<Route> {
        match key {
            KeyCode::Char('s') | KeyCode::Char('c') => {
                if self.ctrl {
                    // TODO: go back
                    return Some(Route::Review);
                }
            }
            KeyCode::Modifier(ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl) => {
                println!("oyoyoy");
                self.ctrl = true;
            }
            _ => self.ctrl = false,
        }

        None
    }
}

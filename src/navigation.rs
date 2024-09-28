use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard,
}

pub struct Review;

impl Review {
    pub const fn new() -> Self {
        Self
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("review...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent) -> Option<Route> {
        if let KeyCode::Char('e') = key.code {
            return Some(Route::EditCard);
        }

        None
    }
}

pub struct AddCard;

impl AddCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("add card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent) -> Option<Route> {
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
}

pub struct EditCard;

impl EditCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn render(&self, body: Rect, buf: &mut Buffer) {
        Paragraph::new("edit card...").render(body, buf);
    }

    pub fn input(&mut self, key: KeyEvent) -> Option<Route> {
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
}

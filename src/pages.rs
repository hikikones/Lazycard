use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::{app::Action, database::*, editor::*, markup::*, utils::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<CardId>),
}

impl Route {
    pub const fn title(self) -> &'static str {
        match self {
            Route::Review => "Review",
            Route::Editor(id) => match id {
                Some(_) => "Edit Card",
                None => "New Card",
            },
        }
    }
}

pub struct Pages {
    pub review: Review,
    pub editor: CardEditor,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: Review::new(),
            editor: CardEditor::new(),
        }
    }
}

pub struct Review {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    state: ReviewState,
    text: String,
}

enum ReviewState {
    None,
    Review(CardId),
    Done,
}

impl Review {
    pub const fn new() -> Self {
        Self {
            due: Vec::new(),
            total: 0,
            progress: 0,
            state: ReviewState::None,
            text: String::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().rev().map(|(id, _)| id));
        self.total = self.due.len();
        if let Some(id) = self.due.pop() {
            let card = db.get(&id).unwrap();
            self.state = ReviewState::Review(id);
            self.text.push_str(card.0.as_str());
        }
    }

    pub fn on_render(&mut self, area: Rect, buf: &mut Buffer, markup: &mut Markup) {
        match self.state {
            ReviewState::None => {
                Line::raw("no cards to review...")
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
            ReviewState::Review(_) => {
                markup.render_markup(&self.text, area, buf);
            }
            ReviewState::Done => {
                Line::raw("done")
                    .alignment(Alignment::Center)
                    .render(area, buf);
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        match self.state {
            ReviewState::Review(id) => {
                match key {
                    KeyCode::Esc => return Action::Quit,
                    KeyCode::Tab => return Action::Route(Route::Editor(None)),
                    KeyCode::Char('e') => return Action::Route(Route::Editor(Some(id))),
                    KeyCode::Delete => {
                        db.remove(&id);
                        if let Some(next_id) = self.due.pop() {
                            self.state = ReviewState::Review(next_id);
                            let card = db.get(&next_id).unwrap();
                            self.text.clear();
                            self.text.push_str(card.0.as_str());
                        } else {
                            self.state = ReviewState::Done;
                        }
                        return Action::Render;
                    }
                    KeyCode::Char(' ') => {
                        // todo: show answer
                    }
                    KeyCode::Up => {
                        // todo: successful recall
                        // fixme: activates when scrolling with touchpad?
                        if markup.scroll(ScrollMove::Up) {
                            return Action::Render;
                        }
                    }
                    KeyCode::Down => {
                        // todo: unsuccessful recall
                        // fixme: activates when scrolling with touchpad?
                        if markup.scroll(ScrollMove::Down) {
                            return Action::Render;
                        }
                    }
                    KeyCode::Right => {
                        if let Some(next_id) = self.due.pop() {
                            self.due.insert(0, id);
                            self.state = ReviewState::Review(next_id);
                            let card = db.get(&next_id).unwrap();
                            self.text.clear();
                            self.text.push_str(card.0.as_str());
                            return Action::Render;
                        }
                    }
                    _ => {}
                }
            }
            ReviewState::None | ReviewState::Done => match key {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::Editor(None)),
                _ => {}
            },
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.total = 0;
        self.progress = 0;
        self.state = ReviewState::None;
        self.text.clear();
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        match self.state {
            ReviewState::Review(_) => {
                if self.due.is_empty() {
                    &[
                        SHORTCUT_SCROLL,
                        SHORTCUT_EDIT,
                        SHORTCUT_DELETE,
                        SHORTCUT_MENU,
                        SHORTCUT_QUIT,
                    ]
                } else {
                    &[
                        SHORTCUT_SCROLL,
                        SHORTCUT_EDIT,
                        SHORTCUT_DELETE,
                        SHORTCUT_SKIP,
                        SHORTCUT_MENU,
                        SHORTCUT_QUIT,
                    ]
                }
            }
            ReviewState::None | ReviewState::Done => &[SHORTCUT_MENU, SHORTCUT_QUIT],
        }
    }
}

pub struct CardEditor {
    editor: TextEditor,
    state: CardEditorState,
    preview: bool,
}

enum CardEditorState {
    New,
    Edit(CardId),
}

impl CardEditor {
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new(),
            state: CardEditorState::New,
            preview: false,
        }
    }

    pub fn on_enter(&mut self, id: Option<CardId>, db: &Database) {
        match id {
            Some(id) => {
                let card = db.get(&id).unwrap();
                self.editor.clear();
                self.editor.push_str(card.0.as_str());
                self.editor.move_cursor(CursorMove::Start, false);
                self.state = CardEditorState::Edit(id);
            }
            None => {
                self.state = CardEditorState::New;
            }
        }
    }

    pub fn on_render(&mut self, area: Rect, buf: &mut Buffer, markup: &mut Markup) {
        if self.preview {
            markup.render_markup(self.editor.as_str(), area, buf);
        } else {
            self.editor.render(area, buf);
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let shift = modifiers.contains(KeyModifiers::SHIFT);

        match key {
            KeyCode::Esc => return Action::Quit,
            KeyCode::Tab => match self.state {
                CardEditorState::New => return Action::Route(Route::Review),
                CardEditorState::Edit(_) => return Action::Route(Route::Review), // todo: go back
            },
            KeyCode::Up => {
                if self.preview {
                    if markup.scroll(ScrollMove::Up) {
                        return Action::Render;
                    }
                } else {
                    if self.editor.move_cursor(CursorMove::Up, shift) {
                        return Action::Render;
                    }
                }
            }
            KeyCode::Down => {
                if self.preview {
                    if markup.scroll(ScrollMove::Down) {
                        return Action::Render;
                    }
                } else {
                    if self.editor.move_cursor(CursorMove::Down, shift) {
                        return Action::Render;
                    }
                }
            }
            KeyCode::Char('s') => {
                if ctrl {
                    markup.clear();
                    self.preview = false;

                    match self.state {
                        CardEditorState::New => {
                            let card = Card::new(self.editor.as_str().to_owned());
                            db.add(card);
                            self.editor.clear();
                            return Action::Render;
                        }
                        CardEditorState::Edit(id) => {
                            let card = db.get_mut(&id).unwrap();
                            card.0 = self.editor.as_str().to_owned();
                            self.editor.clear();
                            return Action::Route(Route::Review); // todo: go back
                        }
                    }
                } else if !self.preview {
                    self.editor.push_char('s');
                    return Action::Render;
                }
            }
            KeyCode::Char('p') => {
                if ctrl {
                    self.preview = !self.preview;
                } else if !self.preview {
                    self.editor.push_char('p');
                }
                return Action::Render;
            }
            _ => {
                if !self.preview {
                    if self.editor.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.preview = false;

        if let CardEditorState::Edit(_) = self.state {
            self.editor.clear();
        }
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[
            SHORTCUT_SAVE,
            SHORTCUT_PREVIEW,
            SHORTCUT_MENU,
            SHORTCUT_QUIT,
        ]
    }
}

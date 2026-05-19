use database::{Card, CardId, Database};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};
use widgets::{CursorMove, KittyGraphics, Markup, Shortcut, Shortcuts, TextEditor, TextSegment};

use crate::{
    app::{Action, AppInput},
    pages::Log,
    settings::Colors,
    symbols,
    terminal::Terminal,
};

pub struct CardEditorPage {
    editor: TextEditor,
    state: CardEditorState,
    preview: bool,
}

enum CardEditorState {
    New,
    Edit(CardId),
}

impl CardEditorPage {
    pub fn new(colors: &Colors) -> Self {
        Self {
            editor: TextEditor::new()
                .with_placeholder("Content...")
                .with_colors(colors.primary, colors.neutral),
            state: CardEditorState::New,
            preview: false,
        }
    }

    pub fn on_enter(&mut self, id: Option<CardId>, db: &Database) {
        match id {
            Some(id) => {
                let card = db.get(id).unwrap();
                self.editor.clear();
                self.editor.push_str(card.content.as_str());
                self.editor.move_cursor(CursorMove::Start, false);
                self.state = CardEditorState::Edit(id);
            }
            None => {
                self.state = CardEditorState::New;
            }
        }
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut TextSegment,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        let title = match self.state {
            CardEditorState::New => "New Card",
            CardEditorState::Edit(_) => "Edit Card",
        };
        menu.push_str(title, colors.neutral);

        if self.preview {
            markup.render(area, buf, self.editor.as_str(), kitty);
        } else {
            self.editor.render(area, buf);
        }

        shortcuts.extend([
            Shortcut::new("Save", symbols::ctrl!("s")),
            Shortcut::new("Preview (toggle)", symbols::ctrl!("p")),
            Shortcut::new("Edit", symbols::ctrl!("e")),
        ]);
    }

    pub fn on_input(
        &mut self,
        input: AppInput,
        markup: &mut Markup,
        db: &mut Database,
        terminal: &mut Terminal,
    ) -> Action {
        let (key, modifiers) = input.key_pressed_and_modifiers();
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        match key {
            KeyCode::Char('e') => {
                if ctrl {
                    match terminal.temp_leave(|| edit::edit(self.editor.as_str())) {
                        Ok(content) => {
                            self.editor.clear();
                            self.editor.push_str(&content);
                            self.editor.move_cursor(CursorMove::Start, false);
                            return Action::Render;
                        }
                        Err(err) => {
                            return Action::Log(Log::new(err));
                        }
                    }
                }
            }
            KeyCode::Char('p') => {
                if ctrl {
                    self.preview = !self.preview;
                    return Action::Render;
                } else if !self.preview {
                    self.editor.push_char('p');
                    return Action::Render;
                }
            }
            KeyCode::Char('s') => {
                if ctrl {
                    if !self.editor.is_empty() {
                        self.save(db);
                        markup.clear();
                        return Action::Render;
                    }
                } else if !self.preview {
                    self.editor.push_char('s');
                    return Action::Render;
                }
            }
            _ => {
                if self.preview {
                    if markup.input(key) {
                        return Action::Render;
                    }
                } else if self.editor.input(key, modifiers) {
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        if let CardEditorState::Edit(_) = self.state {
            self.editor.clear();
        }
    }

    fn save(&mut self, db: &mut Database) {
        match self.state {
            CardEditorState::New => {
                let card = Card::new(self.editor.as_str());
                db.add(card);
            }
            CardEditorState::Edit(id) => {
                db.update(id, |card| {
                    card.content = self.editor.as_str().to_owned();
                });
            }
        }
        self.state = CardEditorState::New;
        self.preview = false;
        self.editor.clear();
    }
}

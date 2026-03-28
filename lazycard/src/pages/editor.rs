use database::{Card, CardId, Database};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    text::{Line, Span},
};
use widgets::{CursorMove, Markup, Shortcut, ShortcutLine, Shortcuts, TextEditor};

use crate::{app::Action, settings::Colors, terminal::Terminal};

pub struct CardEditorPage {
    editor: TextEditor,
    state: CardEditorState,
    preview: bool,
    external_editor: bool,
}

enum CardEditorState {
    New,
    Edit(CardId),
}

impl CardEditorPage {
    pub fn new(external_editor: bool, colors: &Colors) -> Self {
        Self {
            editor: TextEditor::new()
                .with_placeholder("content...")
                .with_colors(colors.accent, colors.neutral),
            state: CardEditorState::New,
            preview: false,
            external_editor,
        }
    }

    pub fn on_enter(
        &mut self,
        id: Option<CardId>,
        db: &Database,
        terminal: &mut Terminal,
    ) -> std::io::Result<()> {
        self.preview = self.external_editor;

        match id {
            Some(id) => {
                let card = db.get(id).unwrap();
                self.editor.clear();
                self.editor.push_str(card.content.as_str());
                self.editor.move_cursor(CursorMove::Start, false);
                self.state = CardEditorState::Edit(id);

                if self.external_editor {
                    let content = terminal.temp_leave(|| edit::edit(self.editor.as_str()))?;
                    self.editor.clear();
                    self.editor.push_str(&content);
                }
            }
            None => {
                self.state = CardEditorState::New;
            }
        }

        Ok(())
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
        let title = match self.state {
            CardEditorState::New => "New Card",
            CardEditorState::Edit(_) => "Edit Card",
        };
        menu.push_span(Span::styled(title, colors.neutral));

        if self.preview {
            markup.render(self.editor.as_str(), area, buf);
            shortcuts.extend(ShortcutLine::Middle, Markup::SHORTCUTS);
        } else {
            self.editor.render(area, buf);
            shortcuts.extend(ShortcutLine::Middle, TextEditor::SHORTCUTS);
        }

        shortcuts.extend(
            ShortcutLine::Top,
            [
                Shortcut::new("Save", "^s"),
                if self.external_editor {
                    Shortcut::new("Edit", "e")
                } else {
                    Shortcut::new("Toggle preview", "^p")
                },
            ],
        );
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
        terminal: &mut Terminal,
    ) -> std::io::Result<Action> {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let shift = modifiers.contains(KeyModifiers::SHIFT);

        if key == KeyCode::Char('p') && ctrl && !self.external_editor {
            self.preview = !self.preview;
            return Ok(Action::Render);
        }

        if self.preview {
            match key {
                KeyCode::Char('e') => {
                    if self.external_editor {
                        let content = terminal.temp_leave(|| edit::edit(self.editor.as_str()))?;
                        self.editor.clear();
                        self.editor.push_str(&content);
                        return Ok(Action::Render);
                    }
                }
                KeyCode::Char('s') => {
                    if ctrl && !self.editor.is_empty() {
                        self.save(db);
                        markup.clear();
                        return Ok(Action::Render);
                    }
                }
                _ => {
                    if markup.input(key, modifiers) {
                        return Ok(Action::Render);
                    }
                }
            }
        } else {
            match key {
                KeyCode::Up => {
                    if self.editor.move_cursor(CursorMove::Up, shift) {
                        return Ok(Action::Render);
                    }
                }
                KeyCode::Down => {
                    if self.editor.move_cursor(CursorMove::Down, shift) {
                        return Ok(Action::Render);
                    }
                }
                KeyCode::Char('s') => {
                    if ctrl {
                        if !self.editor.is_empty() {
                            self.save(db);
                            markup.clear();
                        }
                    } else {
                        self.editor.push_char('s');
                    }
                    return Ok(Action::Render);
                }
                KeyCode::Char('p') => {
                    self.editor.push_char('p');
                    return Ok(Action::Render);
                }
                _ => {
                    if self.editor.input(key, modifiers) {
                        return Ok(Action::Render);
                    }
                }
            }
        }

        Ok(Action::None)
    }

    pub fn on_exit(&mut self) {
        if let CardEditorState::Edit(_) = self.state {
            self.editor.clear();
        }
    }

    fn save(&mut self, db: &mut Database) {
        self.preview = self.external_editor;
        match self.state {
            CardEditorState::New => {
                let card = Card::new(self.editor.as_str().to_owned());
                db.add(card);
            }
            CardEditorState::Edit(id) => {
                db.update(id, |card| {
                    card.content = self.editor.as_str().to_owned();
                    self.state = CardEditorState::New;
                });
            }
        }
        self.editor.clear();
    }
}

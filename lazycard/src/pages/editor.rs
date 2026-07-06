use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Style},
};
use widgets::{CursorMove, KittyGraphics, List, ListItem, Markup, Shortcut, Shortcuts, TextEditor};

use crate::{
    app::{Action, AppInput, AppRender},
    database::{CardId, Database, TagId},
    pages::Log,
    settings::Colors,
    symbols,
    terminal::Terminal,
};

pub struct CardEditorPage {
    editor: TextEditor,
    preview: bool,
    show_tags: bool,
    tags: TagsSidebar,
    card: Option<CardId>,
}

impl CardEditorPage {
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new()
                .with_placeholder("Content...")
                .with_margins(2, 2),
            preview: false,
            show_tags: false,
            tags: TagsSidebar::new(),
            card: None,
        }
    }

    pub fn on_enter(&mut self, id: Option<CardId>, db: &Database) {
        self.preview = false;
        self.tags.update(db);
        self.card = id;

        if let Some(id) = id {
            self.editor.clear();
            db.get_card_content(id, |content| {
                self.editor.push_str(content);
                self.editor.move_cursor(CursorMove::Start, false);
                self.tags.reset_toggles(db, Some(id));
            });
        }
    }

    pub fn on_render(
        &mut self,
        render: AppRender,
        db: &Database,
        colors: &Colors,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        let (mut area, buf) = render.area_and_buffer();

        if self.show_tags {
            let tags_width = (0.30 * area.width as f32).round() as u16;
            self.tags.render(
                Rect {
                    width: tags_width,
                    ..area
                },
                buf,
                db,
                colors,
            );
            area.width = area.width.saturating_sub(tags_width);
            area.x += tags_width + 2;

            if !self.tags.is_empty() {
                shortcuts.extend([
                    Shortcut::new("Toggle", symbols::SPACE),
                    Shortcut::new("Reset", "r"),
                ]);
            }
        }

        let title = if self.card.is_some() {
            "Edit Card"
        } else {
            "New Card"
        };
        widgets::print_ascii(
            area,
            buf,
            title,
            colors.neutral,
            Some(widgets::Alignment::CenterHorizontal),
        );

        area.height = area.height.saturating_sub(2);
        area.y += 2;

        if area.height > 0 {
            if self.preview {
                markup.render(area, buf, self.editor.as_str(), kitty);
            } else {
                self.editor
                    .set_colors(colors.text_editor())
                    .render(area, buf);
            }
        }

        shortcuts.extend([
            Shortcut::new("Save", symbols::ctrl!("s")),
            Shortcut::new("Preview", symbols::ctrl!("p")),
            Shortcut::new("Tags", symbols::ctrl!("t")),
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
                    match terminal.temp_leave(|| {
                        edit::edit_with_builder(
                            self.editor.as_str(),
                            edit::Builder::new().suffix(".md"),
                        )
                    }) {
                        Ok(content) => {
                            self.editor.clear();
                            self.editor.push_str(&content);
                            self.editor.move_cursor(CursorMove::Start, false);
                            return Action::Render;
                        }
                        Err(err) => {
                            return Action::EnqueueLog(Log::new(err));
                        }
                    }
                } else if !self.show_tags && !self.preview {
                    self.editor.push_char('e');
                    return Action::Render;
                }
            }
            KeyCode::Char('p') => {
                if ctrl {
                    self.preview = !self.preview;
                    return Action::Render;
                } else if !self.show_tags && !self.preview {
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
                } else if !self.show_tags && !self.preview {
                    self.editor.push_char('s');
                    return Action::Render;
                }
            }
            KeyCode::Char('t') => {
                if ctrl {
                    self.show_tags = !self.show_tags;
                } else if !self.show_tags && !self.preview {
                    self.editor.push_char('t');
                }
                return Action::Render;
            }
            KeyCode::Char(' ') => {
                if self.show_tags {
                    if self.tags.toggle_selection() {
                        return Action::Render;
                    }
                } else if !self.preview {
                    self.editor.push_char(' ');
                    return Action::Render;
                }
            }
            KeyCode::Char('r') => {
                if self.show_tags {
                    if self.tags.reset_toggles(db, self.card) {
                        return Action::Render;
                    }
                } else if !self.preview {
                    self.editor.push_char('r');
                    return Action::Render;
                }
            }
            _ => {
                if self.show_tags {
                    if self.tags.list.input(key, modifiers) {
                        return Action::Render;
                    }
                } else if self.preview {
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
        if self.card.is_some() {
            self.editor.clear();
            self.show_tags = false;
            self.tags.clear_toggles();
        }
    }

    fn save(&mut self, db: &Database) {
        let cid = match self.card.take() {
            Some(cid) => {
                db.update_card(cid, self.editor.as_str());

                // Remove tags
                for tid in self.tags.to_remove() {
                    db.delete_tag_for_card(cid, tid);
                }

                cid
            }
            None => db.add_card(self.editor.as_str()),
        };

        // Add tags
        for tid in self.tags.to_add() {
            db.add_tag_for_card(cid, tid);
        }

        self.preview = false;
        self.show_tags = false;
        self.editor.clear();
        self.tags.clear_toggles();
    }
}

struct TagsSidebar {
    tags: Vec<TagId>,
    toggles: HashMap<TagId, TagState>,
    list: List,
}

#[derive(Debug, Clone, Copy)]
enum TagState {
    Add,
    Keep,
    Remove,
}

impl TagsSidebar {
    fn new() -> Self {
        Self {
            tags: Vec::new(),
            toggles: HashMap::new(),
            list: List::new(),
        }
    }

    fn update(&mut self, db: &Database) {
        self.tags.clear();
        db.get_tags(|tid| self.tags.push(tid));
    }

    const fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    fn to_add(&self) -> impl Iterator<Item = TagId> {
        self.toggles
            .iter()
            .filter(|(_, state)| matches!(state, TagState::Add))
            .map(|(id, _)| *id)
    }

    fn to_remove(&self) -> impl Iterator<Item = TagId> {
        self.toggles
            .iter()
            .filter(|(_, state)| matches!(state, TagState::Remove))
            .map(|(id, _)| *id)
    }

    fn toggle(&mut self, id: TagId) {
        match self.toggles.get_mut(&id) {
            Some(state) => match state {
                TagState::Add => {
                    self.toggles.remove(&id);
                }
                TagState::Keep => {
                    *state = TagState::Remove;
                }
                TagState::Remove => {
                    *state = TagState::Keep;
                }
            },
            None => {
                self.toggles.insert(id, TagState::Add);
            }
        }
    }

    fn toggle_selection(&mut self) -> bool {
        if self.tags.is_empty() {
            return false;
        }

        for i in self.list.selection_inclusive() {
            let id = self.tags[i];
            self.toggle(id);
        }

        true
    }

    fn reset_toggles(&mut self, db: &Database, cid: Option<CardId>) -> bool {
        let is_empty = self.toggles.is_empty();
        self.toggles.clear();

        if let Some(cid) = cid {
            db.get_tags_for_card(cid, |tid| {
                self.toggles.insert(tid, TagState::Keep);
            });
        }

        !is_empty
    }

    fn clear_toggles(&mut self) {
        self.toggles.clear();
    }

    fn render(&mut self, mut area: Rect, buf: &mut Buffer, db: &Database, colors: &Colors) {
        widgets::print_ascii(
            area,
            buf,
            "Tags",
            Style::new(),
            Some(widgets::Alignment::CenterHorizontal),
        );

        if self.tags.is_empty() {
            widgets::print_ascii(
                area,
                buf,
                "No tags",
                Style::new(),
                Some(widgets::Alignment::CenterHorizontal),
            );
            return;
        }

        area.y += 2;
        area.height = area.height.saturating_sub(2);

        // Render tags
        self.list.set_scrollbar(colors.scrollbar()).render(
            area,
            buf,
            self.tags.iter().copied(),
            |line, buf, id, item| {
                let symbol = match item {
                    ListItem::Selected => symbols::concat!(symbols::SELECTED, " "),
                    ListItem::Selection => symbols::concat!(symbols::SELECTION, " "),
                    ListItem::Normal => "",
                };
                let color = self
                    .toggles
                    .get(&id)
                    .map(|state| match state {
                        TagState::Add | TagState::Keep => Color::Green,
                        TagState::Remove => Color::Red,
                    })
                    .unwrap_or(Color::Reset);

                db.get_tag_name(id, |name| {
                    widgets::print_texts(line, buf, [symbol, name], color, false, None);
                });
            },
        );
    }
}

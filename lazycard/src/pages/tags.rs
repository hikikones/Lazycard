use std::ops::Range;

use database::{Database, TagId};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Widget,
};
use utils::Formatter;
use widgets::{Shortcut, Shortcuts, TextInput, TextSegment, TokenItem, TokenList};

use crate::{
    app::{Action, AppInput},
    settings::Colors,
    symbols,
};

// TODO: Scrollbar.
// TODO: Ignore case-sensitivity?

pub struct TagsPage {
    tags: Vec<TagItem>,
    names: Formatter,
    list: TokenList,
    state: State,
    input: TextInput,
    message: String,
}

enum State {
    Browse,
    New,
    Edit(TagId),
    Delete(TagId),
}

impl TagsPage {
    pub fn new(db: &Database, colors: &Colors) -> Self {
        let mut tags = Vec::new();
        let mut names = Formatter::new();

        db.get_tags_and_name(|id, name| {
            tags.push(TagItem {
                id,
                name: names.push_str(name),
                width: unicode_width::UnicodeWidthStr::width(name) as u16,
            });
        })
        .unwrap();

        let mut tags_page = Self {
            tags,
            names,
            list: TokenList::new(),
            state: State::Browse,
            input: TextInput::new()
                .with_placeholder("Tag name...")
                .with_colors(colors.text_input()),
            message: String::new(),
        };
        tags_page.sort();
        tags_page
    }

    pub fn on_enter(&self) {}

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        menu: &mut TextSegment,
        shortcuts: &mut Shortcuts,
    ) {
        menu.push_str("Tags", colors.neutral);

        match self.state {
            State::Browse => {
                if self.tags.is_empty() {
                    widgets::print_ascii(
                        area,
                        buf,
                        "No tags",
                        colors.neutral,
                        Some(widgets::Alignment::Center),
                    );
                    shortcuts.push(Shortcut::new("New", "n"));
                    return;
                }

                self.list.render(
                    area,
                    buf,
                    self.tags.iter(),
                    |area, buf, tag, is_selected| {
                        let style = if is_selected {
                            Style::new().fg(colors.primary)
                        } else {
                            Style::new()
                        };
                        let name = self.names.slice(tag.name.clone());
                        Span::styled(name, style).render(area, buf);
                    },
                );

                shortcuts.extend([
                    Shortcut::new("New", "n"),
                    Shortcut::new("Edit", "e"),
                    Shortcut::new("Delete", symbols::DELETE),
                ]);
            }
            State::New => {
                let mut area = widgets::align(
                    Rect {
                        width: area.width / 2,
                        height: 5,
                        ..area
                    },
                    area,
                    widgets::Alignment::CenterHorizontal,
                );

                widgets::print_ascii(
                    area,
                    buf,
                    "Create a new tag",
                    Style::new(),
                    Some(widgets::Alignment::CenterHorizontal),
                );

                area.y += 2;
                area.height -= 2;

                self.input.render(area, buf);

                area.y += 2;
                area.height -= 2;

                if !self.message.is_empty() {
                    widgets::print_text(
                        area,
                        buf,
                        self.message.as_str(),
                        Color::Red,
                        false,
                        Some(widgets::Alignment::CenterHorizontal),
                    );
                }

                shortcuts.extend([
                    Shortcut::new("Confirm", symbols::ENTER),
                    Shortcut::new("Cancel", symbols::ctrl!("c")),
                ]);
            }
            State::Edit(id) => {
                let mut area = widgets::align(
                    Rect {
                        width: area.width / 2,
                        height: 5,
                        ..area
                    },
                    area,
                    widgets::Alignment::CenterHorizontal,
                );

                let tag_name = self.names.slice(self.tags[self.list.index()].name.clone());
                widgets::print_asciis(
                    area,
                    buf,
                    ["Edit tag '", tag_name, "'"],
                    Style::new(),
                    Some(widgets::Alignment::CenterHorizontal),
                );

                area.y += 2;
                area.height -= 2;

                self.input.render(area, buf);

                area.y += 2;
                area.height -= 2;

                if !self.message.is_empty() {
                    widgets::print_text(
                        area,
                        buf,
                        self.message.as_str(),
                        Color::Red,
                        false,
                        Some(widgets::Alignment::CenterHorizontal),
                    );
                }

                shortcuts.extend([
                    Shortcut::new("Confirm", symbols::ENTER),
                    Shortcut::new("Cancel", symbols::ctrl!("c")),
                ]);
            }
            State::Delete(id) => {
                //todo
            }
        }
    }

    pub fn on_input(&mut self, input: AppInput, db: &Database) -> Action {
        let (key, modifiers) = input.key_pressed_and_modifiers();

        match self.state {
            State::Browse => match key {
                KeyCode::Delete => {
                    if let Some(id) = self.current_tag_id() {
                        self.state = State::Delete(id);
                        return Action::Render;
                    }
                }
                KeyCode::Char('n') => {
                    self.state = State::New;
                    return Action::Render;
                }
                KeyCode::Char('e') => {
                    if let Some(id) = self.current_tag_id() {
                        self.state = State::Edit(id);
                        return Action::Render;
                    }
                }
                _ => {
                    if self.list.input(key, self.tags.iter()) {
                        return Action::Render;
                    }
                }
            },
            State::New => match key {
                KeyCode::Enter => {
                    // Confirm tag creation
                    let name = self.input.as_str_trim();
                    if !name.is_empty() {
                        match db.add_tag(name).unwrap() {
                            Some(id) => {
                                self.tags.push(TagItem {
                                    id,
                                    name: self.names.push_str(name),
                                    width: unicode_width::UnicodeWidthStr::width(name) as u16,
                                });
                                self.sort();
                                self.select_tag(id);
                                self.input.clear();
                                self.message.clear();
                                self.state = State::Browse;
                            }
                            None => {
                                self.message.clear();
                                self.message
                                    .extend(["Tag name '", name, "' already exists"]);
                            }
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Char('c') => {
                    // Cancel tag creation
                    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                    if ctrl {
                        self.input.clear();
                        self.message.clear();
                        self.state = State::Browse;
                    } else {
                        self.input.push_char('c');
                    }
                    return Action::Render;
                }
                _ => {
                    if self.input.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            },
            State::Edit(id) => match key {
                KeyCode::Enter => {
                    // Confirm tag edit
                    let name = self.input.as_str_trim();
                    if !name.is_empty() {
                        if db.update_tag(id, name).unwrap() {
                            let tag = &mut self.tags[self.list.index()];
                            tag.name = self.names.push_str(name);
                            tag.width = unicode_width::UnicodeWidthStr::width(name) as u16;
                            self.sort();
                            self.select_tag(id);
                            self.input.clear();
                            self.message.clear();
                            self.state = State::Browse;
                        } else {
                            self.message.clear();
                            self.message
                                .extend(["Tag name '", name, "' already exists"]);
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Char('c') => {
                    // Cancel tag edit
                    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                    if ctrl {
                        self.input.clear();
                        self.message.clear();
                        self.state = State::Browse;
                    } else {
                        self.input.push_char('c');
                    }
                    return Action::Render;
                }
                _ => {
                    if self.input.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            },
            State::Delete(id) => {
                //todo
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.tags.clear();
        self.names.clear();
        self.message.clear();
    }

    fn current_tag_id(&self) -> Option<TagId> {
        self.tags.get(self.list.index()).map(|tag| tag.id)
    }

    fn select_tag(&mut self, id: TagId) {
        if let Some(i) = self
            .tags
            .iter()
            .enumerate()
            .find(|(_, tag)| tag.id == id)
            .map(|(i, _)| i)
        {
            self.list.set_index(i);
        }
    }

    fn sort(&mut self) {
        self.tags.sort_unstable_by(|t1, t2| {
            let n1 = self.names.slice(t1.name.clone());
            let n2 = self.names.slice(t2.name.clone());
            n1.cmp(n2)
        });
    }
}

struct TagItem {
    id: TagId,
    name: Range<usize>,
    width: u16,
}

impl TokenItem for &TagItem {
    fn width(&self) -> u16 {
        self.width
    }
}

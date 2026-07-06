use std::ops::Range;

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Widget,
};
use utils::Formatter;
use widgets::{Shortcut, Shortcuts, TextInput, TokenItem, TokenList};

use crate::{
    app::{Action, AppInput, AppRender},
    database::{Database, TagId},
    pages::{CardsParam, Route},
    settings::Colors,
    symbols,
};

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
    Delete(TagId, bool),
}

impl TagsPage {
    pub fn new(db: &Database) -> Self {
        let mut tags = Vec::new();
        let mut names = Formatter::new();

        db.get_tags_and_name(|id, name| tags.push(TagItem::new(id, name, &mut names)));

        let mut tags_page = Self {
            tags,
            names,
            list: TokenList::new(),
            state: State::Browse,
            input: TextInput::new().with_placeholder("Tag name..."),
            message: String::new(),
        };
        tags_page.sort();
        tags_page
    }

    pub fn on_enter(&self) {}

    pub fn on_render(&mut self, render: AppRender, colors: &Colors, shortcuts: &mut Shortcuts) {
        let (mut area, buf) = render.area_and_buffer();

        utils::format_int(self.tags.len(), |tags_len| {
            widgets::print_asciis(
                area,
                buf,
                ["Tags (", tags_len, ")"],
                colors.neutral,
                Some(widgets::Alignment::CenterHorizontal),
            );
        });

        area.height = area.height.saturating_sub(2);
        area.y += 2;

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

                // Render tags
                self.list.set_scrollbar(colors.scrollbar()).render(
                    area,
                    buf,
                    self.tags.iter(),
                    |area, buf, tag, is_selected| {
                        let style = if is_selected {
                            Style::new().fg(colors.secondary)
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
                    Shortcut::new("Goto", "g"),
                    Shortcut::new("Delete", symbols::DELETE),
                ]);
            }
            State::New => {
                widgets::print_ascii(
                    area,
                    buf,
                    "New tag",
                    Style::new(),
                    Some(widgets::Alignment::CenterHorizontal),
                );

                area.height = area.height.saturating_sub(2);
                area.y += 2;

                if area.height > 0 {
                    let input_area = widgets::align(
                        Rect {
                            width: (0.64 * area.width as f32).round() as u16,
                            height: 1,
                            ..area
                        },
                        area,
                        widgets::Alignment::CenterHorizontal,
                    );
                    self.input
                        .set_colors(colors.text_input())
                        .render(input_area, buf);

                    area.height = area.height.saturating_sub(2);
                    area.y += 2;

                    if area.height > 0 && !self.message.is_empty() {
                        widgets::print_text(
                            area,
                            buf,
                            self.message.as_str(),
                            Color::Red,
                            false,
                            Some(widgets::Alignment::CenterHorizontal),
                        );
                    }
                }

                shortcuts.extend([
                    Shortcut::new("Confirm", symbols::ENTER),
                    Shortcut::new("Cancel", symbols::ctrl!("c")),
                ]);
            }
            State::Edit(_id) => {
                widgets::print_ascii(
                    area,
                    buf,
                    "Edit tag",
                    Style::new(),
                    Some(widgets::Alignment::CenterHorizontal),
                );

                area.height = area.height.saturating_sub(1);
                area.y += 1;

                if area.height > 0 {
                    widgets::print_text(
                        area,
                        buf,
                        self.current_tag_name(),
                        Style::new(),
                        false,
                        Some(widgets::Alignment::CenterHorizontal),
                    );

                    area.height = area.height.saturating_sub(2);
                    area.y += 2;

                    if area.height > 0 {
                        let input_area = widgets::align(
                            Rect {
                                width: (0.64 * area.width as f32).round() as u16,
                                height: 1,
                                ..area
                            },
                            area,
                            widgets::Alignment::CenterHorizontal,
                        );
                        self.input
                            .set_colors(colors.text_input())
                            .render(input_area, buf);

                        area.height = area.height.saturating_sub(2);
                        area.y += 2;

                        if area.height > 0 && !self.message.is_empty() {
                            widgets::print_text(
                                area,
                                buf,
                                self.message.as_str(),
                                Color::Red,
                                false,
                                Some(widgets::Alignment::CenterHorizontal),
                            );
                        }
                    }
                }

                shortcuts.extend([
                    Shortcut::new("Confirm", symbols::ENTER),
                    Shortcut::new("Cancel", symbols::ctrl!("c")),
                ]);
            }
            State::Delete(_id, delete_cards) => {
                widgets::print_ascii(
                    area,
                    buf,
                    "Delete tag",
                    Style::new(),
                    Some(widgets::Alignment::CenterHorizontal),
                );

                area.height = area.height.saturating_sub(1);
                area.y += 1;

                if area.height > 0 {
                    widgets::print_text(
                        area,
                        buf,
                        self.current_tag_name(),
                        Style::new(),
                        false,
                        Some(widgets::Alignment::CenterHorizontal),
                    );

                    area.height = area.height.saturating_sub(2);
                    area.y += 2;

                    if area.height > 0 {
                        let checkmark = if delete_cards {
                            (symbols::CHECKMARK_YES, Style::new().fg(Color::Green))
                        } else {
                            (symbols::CHECKMARK_NO, Style::new().fg(Color::Red))
                        };
                        widgets::print_texts_with_styles(
                            area,
                            buf,
                            [("Also delete cards: ", Style::new()), checkmark],
                            None,
                            Some(widgets::Alignment::CenterHorizontal),
                        );
                    }
                }

                shortcuts.extend([
                    Shortcut::new("Yes", "y"),
                    Shortcut::new("No", "n"),
                    Shortcut::new("Toggle", symbols::SPACE),
                ]);
            }
        }
    }

    pub fn on_input(&mut self, input: AppInput, db: &Database) -> Action {
        let (key, modifiers) = input.key_pressed_and_modifiers();

        match self.state {
            State::Browse => match key {
                KeyCode::Delete => {
                    if let Some(id) = self.current_tag_id() {
                        self.state = State::Delete(id, false);
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
                KeyCode::Char('g') => {
                    if let Some(id) = self.current_tag_id() {
                        return Action::Route(Route::Cards(Some(CardsParam::Tag(id))));
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
                        match db.add_tag(name) {
                            Some(id) => {
                                self.tags.push(TagItem::new(id, name, &mut self.names));
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
                        if db.update_tag(id, name) {
                            let current_tag = &mut self.tags[self.list.index()];
                            current_tag.update(name, &mut self.names);
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
            State::Delete(id, delete_cards) => {
                match key {
                    KeyCode::Char('y') => {
                        // Delete tag
                        if delete_cards {
                            db.delete_cards_with_tag(id);
                        }

                        db.delete_tag(id);
                        self.tags.remove(self.list.index());
                        self.list
                            .set_index(self.list.index().min(self.tags.len().saturating_sub(1)));

                        self.state = State::Browse;
                        return Action::Render;
                    }
                    KeyCode::Char('n') => {
                        // Cancel delete tag
                        self.state = State::Browse;
                        return Action::Render;
                    }
                    KeyCode::Char(' ') => {
                        // Toggle card deletion
                        self.state = State::Delete(id, !delete_cards);
                        return Action::Render;
                    }
                    _ => {}
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&self) {}

    fn current_tag_id(&self) -> Option<TagId> {
        self.tags.get(self.list.index()).map(|tag| tag.id)
    }

    fn current_tag_name(&self) -> &str {
        self.names.slice(self.tags[self.list.index()].name.clone())
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

impl TagItem {
    fn new(id: TagId, name: &str, formatter: &mut Formatter) -> Self {
        Self {
            id,
            name: formatter.push_str(name),
            width: unicode_width::UnicodeWidthStr::width(name) as u16,
        }
    }

    fn update(&mut self, name: &str, formatter: &mut Formatter) {
        self.name = formatter.push_str(name);
        self.width = unicode_width::UnicodeWidthStr::width(name) as u16;
    }
}

impl TokenItem for &TagItem {
    fn width(&self) -> u16 {
        self.width
    }
}

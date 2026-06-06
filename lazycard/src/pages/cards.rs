use std::collections::HashSet;

use database::{CardId, Database, TagId};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Padding, Widget},
};
use widgets::{
    KittyGraphics, List, ListItem, ListMove, Markup, ScrollMove, Shortcut, Shortcuts, TextInput,
    TextInputColors, TextSegment,
};

use crate::{
    app::{Action, AppInput},
    pages::Route,
    settings::Colors,
    symbols,
};

pub struct CardsPage {
    state: State,
    cards: Vec<CardId>,
    index: usize,
    show_tags: bool,
    tags: TagsSidebar,
}

enum State {
    Search,
    Browse,
}

impl CardsPage {
    pub fn new(colors: &Colors) -> Self {
        Self {
            state: State::Browse,
            cards: Vec::new(),
            index: 0,
            show_tags: false,
            tags: TagsSidebar::new(colors),
        }
    }

    pub fn on_enter(&mut self, db: &mut Database) {
        self.tags.update(db);
        self.update_cards(db);
    }

    pub fn on_render(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        db: &Database,
        colors: &Colors,
        menu: &mut TextSegment,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        let border_color = if self.show_tags {
            let tags_width = ((0.25 * area.width as f32).round() as u16).min(20);
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
            area.x += tags_width;
            colors.neutral
        } else {
            colors.secondary
        };

        let block = Block::bordered()
            .border_style(border_color)
            .padding(Padding::horizontal(1));
        let card_area = block.inner(area);
        block.render(area, buf);

        match self.current_card() {
            Some(id) => {
                let neutral = Style::new().fg(colors.neutral);
                menu.push_int(self.index + 1, neutral);
                menu.push_str(" / ", neutral);
                menu.push_int(self.cards.len(), neutral);

                db.get_card_content(id, |content| {
                    markup.render(card_area, buf, content, kitty);
                })
                .unwrap();

                shortcuts.extend([
                    Shortcut::new("Edit", "e"),
                    Shortcut::new("Delete", symbols::DELETE),
                ]);
            }
            None => {
                widgets::print_ascii(
                    card_area,
                    buf,
                    "No cards",
                    colors.neutral,
                    Some(widgets::Alignment::Center),
                );
            }
        }

        shortcuts.extend([Shortcut::new("Tags", "t"), Shortcut::new("Search", "s")]);
    }

    pub fn on_input(&mut self, input: AppInput, markup: &mut Markup, db: &mut Database) -> Action {
        // if self.cards.is_empty() {
        //     return Action::None;
        // }

        let (key, modifiers) = input.key_pressed_and_modifiers();

        match self.state {
            State::Browse => {
                if self.show_tags {
                    match key {
                        KeyCode::Up => {
                            match self.tags.state {
                                TagsState::Search => {
                                    //todo: move up to card search?
                                }
                                TagsState::Tagless => {
                                    self.tags.state = TagsState::Search;
                                    return Action::Render;
                                }
                                TagsState::Browse => {
                                    if self.tags.list.index() == 0 {
                                        self.tags.state = TagsState::Tagless;
                                    } else {
                                        self.tags.list.move_index(ListMove::Up(1), false);
                                    }
                                    return Action::Render;
                                }
                            }
                        }
                        KeyCode::Down => match self.tags.state {
                            TagsState::Search => {
                                self.tags.state = TagsState::Tagless;
                                return Action::Render;
                            }
                            TagsState::Tagless => {
                                self.tags.state = TagsState::Browse;
                                return Action::Render;
                            }
                            TagsState::Browse => {
                                if self.tags.list.move_index(ListMove::Down(1), false) {
                                    return Action::Render;
                                }
                            }
                        },
                        KeyCode::Right => {
                            return self.next_card(markup);
                        }
                        KeyCode::Left => {
                            return self.previous_card(markup);
                        }
                        KeyCode::Delete => {
                            return self.delete_card(db, markup);
                        }
                        KeyCode::Char('e') => {
                            if let TagsState::Browse = self.tags.state {
                                //todo: edit tag
                            }
                        }
                        KeyCode::Char('s') => {
                            if let TagsState::Search = self.tags.state {
                                self.tags.search.push_char('s');
                            } else {
                                self.tags.state = TagsState::Search;
                            }
                            return Action::Render;
                        }
                        KeyCode::Char('t') => {
                            return self.toggle_tags();
                        }
                        KeyCode::Char(' ') => match self.tags.state {
                            TagsState::Search => {
                                self.tags.search.push_char(' ');
                                return Action::Render;
                            }
                            TagsState::Tagless => {
                                self.tags.tagless = !self.tags.tagless;
                                self.update_cards(db);
                                return Action::Render;
                            }
                            TagsState::Browse => {
                                if let Some(id) = self.tags.current_tag() {
                                    self.tags.toggle(id);
                                    self.update_cards(db);
                                    return Action::Render;
                                }
                            }
                        },
                        _ => {
                            if self.tags.list.input(key, KeyModifiers::empty()) {
                                return Action::Render;
                            }
                        }
                    }
                } else {
                    match key {
                        KeyCode::Right => {
                            return self.next_card(markup);
                        }
                        KeyCode::Left => {
                            return self.previous_card(markup);
                        }
                        KeyCode::Delete => {
                            return self.delete_card(db, markup);
                        }
                        KeyCode::Char('e') => {
                            if let Some(id) = self.current_card() {
                                return Action::Route(Route::Editor(Some(id)));
                            }
                        }
                        KeyCode::Char('s') => {
                            self.state = State::Search;
                            return Action::Render;
                        }
                        KeyCode::Char('t') => {
                            return self.toggle_tags();
                        }
                        _ => {
                            if markup.input(key) {
                                return Action::Render;
                            }
                        }
                    }
                }
            }
            State::Search => todo!(),
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        //todo?
    }

    fn current_card(&self) -> Option<CardId> {
        self.cards.get(self.index).copied()
    }

    fn next_card(&mut self, markup: &mut Markup) -> Action {
        if self.cards.len() > 1 {
            self.index = (self.index + 1) % self.cards.len();
            markup.scroll(ScrollMove::Start);
            return Action::Render;
        }
        Action::None
    }

    fn previous_card(&mut self, markup: &mut Markup) -> Action {
        if self.cards.len() > 1 {
            if self.index == 0 {
                self.index = self.cards.len() - 1;
            } else {
                self.index -= 1;
            }
            markup.scroll(ScrollMove::Start);
            return Action::Render;
        }
        Action::None
    }

    fn delete_card(&mut self, db: &Database, markup: &mut Markup) -> Action {
        if self.index < self.cards.len() {
            let id = self.cards.remove(self.index);
            db.delete_card(id).unwrap();

            if !self.cards.is_empty() {
                self.index = self.index.min(self.cards.len() - 1);
                markup.scroll(ScrollMove::Start);
            }

            return Action::Render;
        }
        Action::None
    }

    fn update_cards(&mut self, db: &mut Database) {
        self.cards.clear();

        if self.tags.tagless {
            db.get_cards_without_tags(&mut self.cards).unwrap();
        } else {
            db.get_cards_with_tags(self.tags.includes(), self.tags.excludes(), &mut self.cards)
                .unwrap();
        }
    }

    fn toggle_tags(&mut self) -> Action {
        self.show_tags = !self.show_tags;
        return Action::Render;
    }
}

pub struct TagsSidebar {
    state: TagsState,
    tags: Vec<TagId>,
    includes: HashSet<TagId>,
    excludes: HashSet<TagId>,
    search: TextInput,
    list: List,
    tagless: bool,
}

#[derive(Debug, Clone, Copy)]
enum TagsState {
    Search,
    Tagless,
    Browse,
}

impl TagsSidebar {
    fn new(colors: &Colors) -> Self {
        Self {
            state: TagsState::Browse,
            tags: Vec::new(),
            includes: HashSet::new(),
            excludes: HashSet::new(),
            search: TextInput::new()
                .with_placeholder("Search...")
                .with_colors(TextInputColors {
                    cursor: colors.primary,
                    ..Default::default()
                }),
            list: List::new(),
            tagless: false,
        }
    }

    fn update(&mut self, db: &Database) {
        self.tags.clear();

        db.get_tags(&mut self.tags).unwrap();

        self.includes.retain(|id| self.tags.contains(id));
        self.excludes.retain(|id| self.tags.contains(id));
    }

    fn current_tag(&self) -> Option<TagId> {
        self.tags.get(self.list.index()).copied()
    }

    fn toggle(&mut self, id: TagId) {
        if self.includes.remove(&id) {
            self.excludes.insert(id);
            return;
        }

        if self.excludes.remove(&id) {
            return;
        }

        self.includes.insert(id);
    }

    fn iter(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.tags.iter().copied()
    }

    fn includes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.includes.iter().copied()
    }

    fn excludes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.excludes.iter().copied()
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, db: &Database, colors: &Colors) {
        let block = Block::bordered()
            .border_style(colors.secondary)
            .title_top(" Tags ")
            .title_style(Style::new().fg(Color::Reset))
            .title_alignment(Alignment::Center)
            .padding(Padding::horizontal(1));
        (&block).render(area, buf);

        // widgets::print_ascii(
        //     area,
        //     buf,
        //     " Tags ",
        //     Style::new(),
        //     Some(widgets::Alignment::CenterHorizontal),
        // );

        let mut area = block.inner(area);
        area.y += 1;
        area.height = area.height.saturating_sub(1);

        //search
        self.search
            .set_disabled(!matches!(self.state, TagsState::Search))
            .render(area, buf);

        area.y += 1;
        area.height = area.height.saturating_sub(1);

        //tagless
        let s = if let TagsState::Tagless = self.state {
            symbols::concat!(symbols::SELECTED, " tagless")
        } else {
            "tagless"
        };
        widgets::print_ascii(area, buf, s, Style::new(), None);

        area.y += 1;
        area.height = area.height.saturating_sub(1);

        //tags
        self.list.set_colors(colors.neutral, None).render(
            area,
            buf,
            self.tags.iter().copied(),
            |line, buf, id, item| {
                if self.tagless {
                    db.get_tag_name(id, |name| {
                        widgets::print_text(line, buf, name, colors.neutral, false, None);
                    })
                    .unwrap();
                    return;
                }

                let symbol = match self.state {
                    TagsState::Browse => {
                        if let ListItem::Selected = item {
                            symbols::concat!(symbols::SELECTED, " ")
                        } else {
                            ""
                        }
                    }
                    TagsState::Search | TagsState::Tagless => "",
                };

                let color = if self.includes.contains(&id) {
                    Color::Green
                } else if self.excludes.contains(&id) {
                    Color::Red
                } else {
                    Color::Reset
                };

                db.get_tag_name(id, |name| {
                    widgets::print_texts(line, buf, [symbol, name], color, false, None);
                })
                .unwrap();
            },
        );
    }
}

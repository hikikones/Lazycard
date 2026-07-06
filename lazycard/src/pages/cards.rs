use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
};
use utils::Formatter;
use widgets::{KittyGraphics, List, ListItem, Markup, ScrollMove, Shortcut, Shortcuts};

use crate::{
    app::{Action, AppInput, AppRender},
    database::{CardId, Database, TagId},
    pages::{CardsParam, Route},
    settings::Colors,
    symbols,
};

pub struct CardsPage {
    cards: Vec<CardId>,
    index: usize,
    show_tags: bool,
    tags: TagsSidebar,
}

impl CardsPage {
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
            show_tags: false,
            tags: TagsSidebar::new(),
        }
    }

    pub fn on_enter(&mut self, db: &mut Database, param: Option<CardsParam>) {
        match param {
            Some(param) => {
                match param {
                    CardsParam::Card(cid) => {
                        self.tags.clear();
                        self.tags.refresh(db);
                        db.get_tags_for_card(cid, |tid| {
                            self.tags.includes.insert(tid);
                        });
                        self.update_cards(db);
                        self.select_card(cid);
                    }
                    CardsParam::Tag(tid) => {
                        self.tags.clear();
                        self.tags.refresh(db);
                        self.tags.includes.insert(tid);
                        self.tags.select(tid);
                        self.update_cards(db);
                    }
                }
                self.show_tags = true;
            }
            None => {
                self.tags.refresh(db);
                self.update_cards(db);
            }
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

        match self.current_card() {
            Some(id) => {
                utils::format_int2(self.index + 1, self.cards.len(), |index, cards_len| {
                    widgets::print_asciis(
                        area,
                        buf,
                        [index, " / ", cards_len],
                        colors.neutral,
                        Some(widgets::Alignment::CenterHorizontal),
                    );
                });

                area.height = area.height.saturating_sub(2);
                area.y += 2;

                if area.height > 0 {
                    db.get_card_content(id, |content| {
                        markup.render(area, buf, content, kitty);
                    });
                }
            }
            None => {
                widgets::print_ascii(
                    area,
                    buf,
                    "No cards",
                    colors.neutral,
                    Some(widgets::Alignment::Center),
                );
            }
        }

        shortcuts.extend([
            Shortcut::new("Edit", "e"),
            Shortcut::new("Delete", symbols::DELETE),
            Shortcut::new("Tags", "t"),
        ]);
    }

    pub fn on_input(&mut self, input: AppInput, markup: &mut Markup, db: &mut Database) -> Action {
        let (key, modifiers) = input.key_pressed_and_modifiers();

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
            KeyCode::Char('t') => {
                return self.toggle_show_tags();
            }
            KeyCode::Char(' ') => {
                if self.show_tags {
                    if self.tags.toggle_selection() {
                        self.update_cards(db);
                        return Action::Render;
                    }
                }
            }
            KeyCode::Char('r') => {
                if self.show_tags && !self.tags.is_empty() {
                    if self.tags.reset() {
                        self.update_cards(db);
                        return Action::Render;
                    }
                }
            }
            _ => {
                if self.show_tags {
                    if self.tags.list.input(key, modifiers) {
                        return Action::Render;
                    }
                } else {
                    if markup.input(key) {
                        return Action::Render;
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&self) {}

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

    fn select_card(&mut self, id: CardId) {
        if let Some(i) = self
            .cards
            .iter()
            .copied()
            .enumerate()
            .find(|(_, cid)| id == *cid)
            .map(|(i, _)| i)
        {
            self.index = i;
        }
    }

    fn delete_card(&mut self, db: &Database, markup: &mut Markup) -> Action {
        if self.index < self.cards.len() {
            let id = self.cards.remove(self.index);
            db.delete_card(id);

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
        self.index = 0;

        db.get_cards_with_tags(self.tags.includes(), self.tags.excludes(), |id| {
            self.cards.push(id);
        });
    }

    fn toggle_show_tags(&mut self) -> Action {
        self.show_tags = !self.show_tags;
        return Action::Render;
    }
}

struct TagsSidebar {
    tags: Vec<TagItem>,
    names: Formatter,
    includes: HashSet<TagId>,
    excludes: HashSet<TagId>,
    list: List,
}

impl TagsSidebar {
    fn new() -> Self {
        Self {
            tags: Vec::new(),
            names: Formatter::new(),
            includes: HashSet::new(),
            excludes: HashSet::new(),
            list: List::new(),
        }
    }

    fn refresh(&mut self, db: &Database) {
        self.tags.clear();
        self.names.clear();

        db.get_tags_and_cards_count(|id, name, cards_count| {
            self.tags
                .push(TagItem::new(id, name, cards_count, &mut self.names))
        });

        self.includes
            .retain(|id| self.tags.iter().any(|tag| tag.id == *id));
        self.excludes
            .retain(|id| self.tags.iter().any(|tag| tag.id == *id));
    }

    const fn is_empty(&self) -> bool {
        self.tags.is_empty()
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

    fn toggle_selection(&mut self) -> bool {
        if self.tags.is_empty() {
            return false;
        }

        for i in self.list.selection_inclusive() {
            let tag = &self.tags[i];
            self.toggle(tag.id);
        }

        true
    }

    fn includes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.includes.iter().copied()
    }

    fn excludes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.excludes.iter().copied()
    }

    fn select(&mut self, id: TagId) {
        if let Some(i) = self
            .tags
            .iter()
            .enumerate()
            .find(|(_, tag)| tag.id == id)
            .map(|(i, _)| i)
        {
            self.list.reset();
            self.list.set_index(i);
        }
    }

    fn reset(&mut self) -> bool {
        let has_includes = !self.includes.is_empty();
        let has_excludes = !self.excludes.is_empty();
        self.includes.clear();
        self.excludes.clear();
        has_includes || has_excludes
    }

    fn clear(&mut self) {
        self.tags.clear();
        self.names.clear();
        self.includes.clear();
        self.excludes.clear();
        self.list.reset();
    }

    fn render(&mut self, mut area: Rect, buf: &mut Buffer, colors: &Colors) {
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
            self.tags.iter(),
            |line, buf, tag, item| {
                let id = tag.id;
                let name = self.names.slice(tag.name.clone());
                let symbol = match item {
                    ListItem::Selected => symbols::concat!(symbols::SELECTED, " "),
                    ListItem::Selection => symbols::concat!(symbols::SELECTION, " "),
                    ListItem::Normal => "",
                };
                let color = if self.includes.contains(&id) {
                    Color::Green
                } else if self.excludes.contains(&id) {
                    Color::Red
                } else {
                    Color::Reset
                };

                widgets::print_texts(line, buf, [symbol, name], color, false, None);
            },
        );
    }
}

struct TagItem {
    id: TagId,
    name: std::ops::Range<usize>,
}

impl TagItem {
    fn new(id: TagId, name: &str, cards_count: u32, formatter: &mut Formatter) -> Self {
        Self {
            id,
            name: formatter.push_fmt(format_args!("{name} ({cards_count})")),
        }
    }
}

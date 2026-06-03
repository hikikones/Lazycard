use database::{CardId, Database};
use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Style,
    widgets::{Block, Padding, Widget},
};
use widgets::{KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextSegment};

use crate::{
    app::{Action, AppInput},
    pages::{Route, TagsList},
    settings::Colors,
    symbols,
};

pub struct CardsPage {
    state: State,
    cards: Vec<CardId>,
    index: usize,
    tags: TagsList,
    show_tags: bool,
}

enum State {
    Browse,
    Search,
}

impl CardsPage {
    pub fn new(colors: &Colors) -> Self {
        Self {
            state: State::Browse,
            cards: Vec::new(),
            index: 0,
            tags: TagsList::new(),
            show_tags: false,
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        db.get_cards(&mut self.cards).unwrap();
        self.tags.on_enter(db);
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
            self.tags.on_render(
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

        match self.cards.get(self.index).copied() {
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
            State::Browse => match key {
                KeyCode::Right => {
                    if self.cards.len() > 1 {
                        self.index = (self.index + 1) % self.cards.len();
                        markup.scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                KeyCode::Left => {
                    if self.cards.len() > 1 {
                        if self.index == 0 {
                            self.index = self.cards.len() - 1;
                        } else {
                            self.index -= 1;
                        }
                        markup.scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                KeyCode::Delete => {
                    let id = self.cards.remove(self.index);
                    db.delete_card(id).unwrap();

                    if !self.cards.is_empty() {
                        self.index = self.index.min(self.cards.len() - 1);
                        markup.scroll(ScrollMove::Start);
                    }

                    return Action::Render;
                }
                KeyCode::Char('e') => {
                    let id = self.cards.get(self.index).copied().unwrap();
                    return Action::Route(Route::Editor(Some(id)));
                }
                KeyCode::Char('s') => {
                    self.state = State::Search;
                    return Action::Render;
                }
                KeyCode::Char('t') => {
                    self.show_tags = !self.show_tags;
                    return Action::Render;
                }
                _ => {
                    let render = if self.show_tags {
                        let r = self.tags.on_input(key);
                        self.index = 0;
                        self.cards.clear();
                        db.get_cards_by_tags(
                            self.tags.includes(),
                            self.tags.excludes(),
                            &mut self.cards,
                        )
                        .unwrap();
                        r
                    } else {
                        markup.input(key)
                    };

                    if render {
                        return Action::Render;
                    }
                }
            },
            State::Search => todo!(),
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.cards.clear();
        self.index = 0;
        self.tags.on_exit();
    }
}

use std::collections::{HashMap, HashSet};

use database::{Database, TagId};
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    prelude::*,
    widgets::{Block, Padding},
};
use widgets::{List, ListItem, ListMove, TextInput};

use crate::{settings::Colors, symbols};

pub struct TagsList {
    state: State,
    tags: Vec<TagId>,
    includes: HashSet<TagId>,
    excludes: HashSet<TagId>,
    // tag_states: TagStates,
    search: TextInput,
    list: List,
    // index: usize,
}

enum State {
    Browse,
    Search,
}

#[derive(Debug, Clone, Copy)]
enum TagState {
    Include,
    Exclude,
}

impl TagsList {
    pub fn new() -> Self {
        Self {
            state: State::Browse,
            tags: Vec::new(),
            includes: HashSet::new(),
            excludes: HashSet::new(),
            // tag_states: TagStates::new(),
            search: TextInput::new().with_placeholder("Search..."),
            list: List::new(),
            // index: 0,
        }
    }

    pub fn init(&mut self, db: &Database) {
        db.get_tags(&mut self.tags).unwrap();
    }

    pub fn includes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.includes.iter().copied()
    }

    pub fn excludes(&self) -> impl ExactSizeIterator<Item = TagId> {
        self.excludes.iter().copied()
    }

    // pub fn includes(&self) -> impl Iterator<Item = TagId> {
    //     self.tag_states
    //         .0
    //         .iter()
    //         .filter(|(_, state)| matches!(state, TagState::Include))
    //         .map(|(id, _)| *id)
    // }

    // pub fn excludes(&self) -> impl Iterator<Item = TagId> {
    //     self.tag_states
    //         .0
    //         .iter()
    //         .filter(|(_, state)| matches!(state, TagState::Exclude))
    //         .map(|(id, _)| *id)
    // }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, db: &Database, colors: &Colors) {
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

        self.list.set_colors(colors.neutral, None).render(
            area,
            buf,
            self.tags.iter().copied(),
            |line, buf, id, item| {
                let symbol = if let ListItem::Selected = item {
                    symbols::concat!(symbols::SELECTED, " ")
                } else {
                    ""
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

    pub fn input(&mut self, key: KeyCode) -> bool {
        match self.state {
            State::Browse => match key {
                KeyCode::Up => {
                    // TODO: move to "tagless" button when index = 0,
                    // or to search bar in already on tagless.
                    // Need to add State::Tagless for this.
                    return self.list.move_index(ListMove::Up(1), false);
                }
                KeyCode::Down => {
                    return self.list.move_index(ListMove::Down(1), false);
                }
                KeyCode::Char(' ') => {
                    if let Some(id) = self.current_tag() {
                        // self.tag_states.toggle(id);
                        self.toggle(id);
                        return true;
                    }
                    // self.state = ListState::Search;
                    // return SearchAction::Render;
                }
                KeyCode::Char('s') => {
                    // todo: search tag names
                }
                _ => {
                    // return self.list.input(key, KeyModifiers::empty());
                    // if markup.input(key) {
                    //     return SearchAction::Render;
                    // }
                }
            },
            State::Search => match key {
                KeyCode::Enter => {
                    // self.tags.clear();
                    // let _ = db.search(self.search.as_str_trim(), &mut self.tags);
                    // if !self.tags.is_empty() {
                    //     self.state = ListState::Browse;
                    // }
                    // return SearchAction::Render;
                }
                KeyCode::Down => {
                    // if !self.tags.is_empty() {
                    //     self.state = ListState::Browse;
                    //     return SearchAction::Render;
                    // }
                }
                KeyCode::Up => {}
                _ => {
                    // if self.search.input(key, modifiers) {
                    //     return SearchAction::Render;
                    // }
                }
            },
        }

        false
    }

    pub fn clear(&mut self) {
        self.state = State::Browse;
        self.tags.clear();
        // self.tag_states.clear();
        self.search.clear();
        self.list.reset();
        // self.index = 0;
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
}

// struct TagStates(HashMap<TagId, TagState>);

// impl TagStates {
//     fn new() -> Self {
//         Self(HashMap::new())
//     }

//     fn color(&self, id: TagId) -> Color {
//         self.0
//             .get(&id)
//             .map(|ts| match ts {
//                 TagState::Include => Color::Green,
//                 TagState::Exclude => Color::Red,
//             })
//             .unwrap_or(Color::Reset)
//     }

//     fn toggle(&mut self, id: TagId) {
//         if self.0.contains_key(&id) {
//             match self.0[&id] {
//                 TagState::Include => {
//                     self.0.insert(id, TagState::Exclude);
//                 }
//                 TagState::Exclude => {
//                     self.0.remove(&id);
//                 }
//             }
//             return;
//         }

//         self.0.insert(id, TagState::Include);
//     }

//     fn clear(&mut self) {
//         self.0.clear();
//     }
// }

use database::{Card, CardId, Database, UnixTime};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};
use widgets::{
    KittyGraphics, Markup, ScrollMove, Shortcut, Shortcuts, TextInput, TextInputColors, TextSegment,
};

use crate::{
    app::{Action, CardsIterExt, Matcher},
    pages::Route,
    settings::Colors,
    symbols,
};

pub struct CardsPage {
    cards: Vec<(CardId, CardStats)>,
    index: usize,
    state: CardState,
    sort: CardSort,
    show_archived: bool,
    search: TextInput,
}

struct CardStats {
    creation: UnixTime,
    difficulty: f32,
    score: u32,
}

impl CardStats {
    const fn new(card: &Card, score: u32) -> Self {
        Self {
            creation: card.creation_time,
            difficulty: card.review_difficulty,
            score,
        }
    }
}

enum CardState {
    Browse,
    Search,
}

enum CardSort {
    Newest,
    Oldest,
    Easy,
    Hard,
    Search,
}

impl CardSort {
    const fn as_str(&self) -> &str {
        match self {
            Self::Newest => "Newest",
            Self::Oldest => "Oldest",
            Self::Easy => "Easy",
            Self::Hard => "Hard",
            Self::Search => "Search",
        }
    }
}

impl CardsPage {
    pub const fn new(colors: &Colors) -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
            state: CardState::Browse,
            sort: CardSort::Newest,
            show_archived: false,
            search: TextInput::new()
                .with_placeholder("Search...")
                .with_colors(TextInputColors {
                    normal: Color::Reset,
                    cursor: colors.primary,
                    selector: colors.neutral,
                    placeholder: colors.neutral,
                    disabled: colors.neutral,
                }),
        }
    }

    fn fetch_cards(&mut self, db: &Database, matcher: &mut Matcher) {
        if self.search.is_empty() {
            let all_cards = db
                .iter()
                .filter(|(_, card)| card.archived == self.show_archived)
                .map(|(id, card)| (id, CardStats::new(card, 0)));
            self.cards.extend(all_cards);
        } else {
            let matched_cards = db
                .iter()
                .filter(|(_, card)| card.archived == self.show_archived)
                .search(self.search.as_str(), matcher)
                .map(|(id, card, score)| (id, CardStats::new(card, score)));
            self.cards.extend(matched_cards);
        }
    }

    fn sort_cards(&mut self) {
        match self.sort {
            CardSort::Newest => {
                self.cards
                    .sort_unstable_by_key(|(_, stats)| std::cmp::Reverse(stats.creation));
            }
            CardSort::Oldest => {
                self.cards.sort_unstable_by_key(|(_, stats)| stats.creation);
            }
            CardSort::Easy => {
                self.cards
                    .sort_unstable_by(|(_, s1), (_, s2)| s1.difficulty.total_cmp(&s2.difficulty));
            }
            CardSort::Hard => {
                self.cards.sort_unstable_by(|(_, s1), (_, s2)| {
                    s1.difficulty.total_cmp(&s2.difficulty).reverse()
                });
            }
            CardSort::Search => {
                self.cards
                    .sort_by_key(|(_, stats)| std::cmp::Reverse(stats.score));
            }
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.cards.extend(
            db.iter()
                .filter(|(_, card)| card.archived == self.show_archived)
                .map(|(id, card)| (id, CardStats::new(card, 0))),
        );
        self.sort_cards();
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
        const SPACING: (&str, Style) = ("    ", Style::new());
        let neutral = Style::new().fg(colors.neutral);

        menu.push_int(self.index + 1, neutral);
        menu.push_str(" / ", neutral);
        menu.push_int(self.cards.len(), neutral);
        menu.extend([
            SPACING,
            (self.sort.as_str(), neutral),
            SPACING,
            (symbols::checkmark(self.show_archived), neutral),
            (" Show archived", neutral),
        ]);

        match self.state {
            CardState::Browse => {
                if !self.search.is_empty() {
                    area.y = area.y.saturating_sub(1);

                    let mut search_line = Line::default().centered();
                    search_line.push_span(Span::styled(
                        self.search.as_str(),
                        Style::new().italic().fg(colors.neutral),
                    ));
                    search_line.render(area, buf);

                    area.y += 2;
                    area.height = area.height.saturating_sub(1);
                }

                match self.cards.get(self.index) {
                    Some((id, _)) => {
                        let card = db.get(*id).unwrap();
                        markup.render(area, buf, card.content.as_str(), kitty);

                        shortcuts.extend([
                            Shortcut::new("Browse", symbols::ARROW_RIGHT_LEFT),
                            Shortcut::new("Search", "/"),
                            Shortcut::new("Sort", "s"),
                            Shortcut::new("Toggle archived", "a"),
                        ]);
                        if !self.cards.is_empty() {
                            shortcuts.push(Shortcut::new("Edit", "e"));
                            if self.show_archived {
                                shortcuts.extend([
                                    Shortcut::new("Restore", "r"),
                                    Shortcut::new("Delete", symbols::DELETE),
                                ]);
                            } else {
                                shortcuts.push(Shortcut::new("Archive", symbols::DELETE));
                            }
                        }
                    }
                    None => {
                        let msg = if db.is_empty() {
                            "You have no cards"
                        } else if self.show_archived && self.search.is_empty() {
                            "You have no archived cards"
                        } else if !self.search.is_empty() {
                            if self.show_archived {
                                "Found no archived cards from search query"
                            } else {
                                "Found no cards from search query"
                            }
                        } else if db.iter().active().count() == 0 {
                            "You have no active cards"
                        } else {
                            "todo: oops no card found"
                        };
                        widgets::print_ascii(
                            area,
                            buf,
                            msg,
                            Style::new(),
                            Some(widgets::Alignment::Center),
                        );

                        shortcuts.extend([
                            Shortcut::new("Search", "/"),
                            Shortcut::new("Sort", "s"),
                            Shortcut::new("Toggle archived", "a"),
                        ]);
                    }
                }
            }
            CardState::Search => {
                let search_area = Rect {
                    height: 1,
                    width: area.width / 2,
                    x: area.x + area.width / 4,
                    y: area.y,
                };
                self.search.render(search_area, buf);
                shortcuts.push(Shortcut::new("Confirm", "↵"));
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
        matcher: &mut Matcher,
    ) -> Action {
        match self.state {
            CardState::Browse => match key {
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
                    if !self.cards.is_empty() {
                        let (id, _) = self.cards.remove(self.index);
                        if self.show_archived {
                            db.remove(id);
                        } else {
                            db.update(id, |card| card.archived = true);
                        }
                        if !self.cards.is_empty() {
                            self.index = self.index.min(self.cards.len() - 1);
                            markup.scroll(ScrollMove::Start);
                        }
                        return Action::Render;
                    }
                }
                KeyCode::Char('e') => {
                    if !self.cards.is_empty() {
                        let (id, _) = self.cards.get(self.index).unwrap();
                        return Action::Route(Route::Editor(Some(*id)));
                    }
                }
                KeyCode::Char('s') => {
                    self.sort = match self.sort {
                        CardSort::Newest => CardSort::Oldest,
                        CardSort::Oldest => CardSort::Easy,
                        CardSort::Easy => CardSort::Hard,
                        CardSort::Hard => {
                            if self.search.is_empty() {
                                CardSort::Newest
                            } else {
                                CardSort::Search
                            }
                        }
                        CardSort::Search => CardSort::Newest,
                    };
                    self.index = 0;
                    self.sort_cards();
                    markup.scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Char('/') => {
                    self.state = CardState::Search;
                    return Action::Render;
                }
                KeyCode::Char('a') => {
                    self.show_archived = !self.show_archived;
                    self.index = 0;
                    self.cards.clear();
                    self.fetch_cards(db, matcher);
                    self.sort_cards();
                    markup.scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Char('r') => {
                    if self.show_archived {
                        let (id, _) = self.cards.remove(self.index);
                        db.update(id, |card| card.archived = false);
                        if !self.cards.is_empty() {
                            self.index = self.index.min(self.cards.len() - 1);
                            markup.scroll(ScrollMove::Start);
                        }
                        return Action::Render;
                    }
                }
                _ => {
                    if markup.input(key) {
                        return Action::Render;
                    }
                }
            },
            CardState::Search => match key {
                KeyCode::Enter => {
                    self.cards.clear();
                    self.index = 0;
                    self.state = CardState::Browse;
                    self.sort = if self.search.is_empty() {
                        CardSort::Newest
                    } else {
                        CardSort::Search
                    };
                    self.fetch_cards(db, matcher);
                    self.sort_cards();
                    return Action::Render;
                }
                _ => {
                    if self.search.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            },
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.cards.clear();
        self.index = 0;
        self.state = CardState::Browse;
        self.sort = CardSort::Newest;
        self.show_archived = false;
        self.search.clear();
    }
}

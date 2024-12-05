use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::{
    app::{Action, Colors},
    database::*,
    editor::*,
    markup::*,
    utils::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<CardId>),
    Cards,
}

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage,
    pub cards: CardsPage,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(),
            cards: CardsPage::new(),
        }
    }
}

pub struct ReviewPage {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    state: ReviewState,
    reveals: Vec<String>,
    text: String,
}

enum ReviewState {
    None,
    Review(CardId),
    Done,
}

impl ReviewPage {
    pub const fn new() -> Self {
        Self {
            due: Vec::new(),
            total: 0,
            progress: 0,
            state: ReviewState::None,
            reveals: Vec::new(),
            text: String::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().is_due().map(|(id, _)| id));
        self.total = self.due.len();

        if !self.due.is_empty() {
            self.next_card(db);
        }
    }

    fn next_card(&mut self, db: &Database) {
        self.reveals.clear();
        self.text.clear();

        if let Some(id) = self.due.pop() {
            let card_content = db.get(id).unwrap().content.as_str();

            let mut start = 0;
            BreakParser::new(card_content).for_each(|i| {
                self.reveals.push(card_content[start..i].to_owned());
                start = i;
            });
            self.reveals.push(card_content[start..].to_owned());
            self.reveals.reverse();

            self.reveal_next();
            self.state = ReviewState::Review(id);
        } else {
            self.state = ReviewState::Done;
        }
    }

    fn reveal_next(&mut self) {
        if let Some(s) = self.reveals.pop() {
            self.text.push_str(s.as_str());
        }
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        menu: &mut Line,
        colors: &Colors,
        markup: &mut Markup,
        shortcuts: &mut Shortcuts,
    ) {
        match self.state {
            ReviewState::None => {
                Line::raw("no cards to review...")
                    .centered()
                    .render(area, buf);
            }
            ReviewState::Review(_) => {
                menu.push_span(Span::styled(
                    format!("{} / {}", self.progress, self.total),
                    STYLE_NONE.fg(colors.neutral),
                ));

                markup.render(&self.text, area, buf, colors);

                if !self.reveals.is_empty() {
                    shortcuts.extend([Shortcut::new("Show", "Space")]);
                } else {
                    shortcuts.extend([Shortcut::new("Yes", "y"), Shortcut::new("No", "n")]);
                }
                if !self.due.is_empty() {
                    shortcuts.extend([Shortcut::new("Skip", "➝")]);
                }
                shortcuts.extend([Shortcut::new("Edit", "e"), Shortcut::new("Delete", "Del")]);
            }
            ReviewState::Done => {
                Line::raw("done").centered().render(area, buf);
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        match self.state {
            ReviewState::Review(id) => match key {
                KeyCode::Char('e') => return Action::Route(Route::Editor(Some(id))),
                KeyCode::Delete => {
                    db.remove(id);
                    self.total = self.total.saturating_sub(1);
                    self.next_card(db);
                    markup.desired_scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Char(' ') => {
                    if !self.reveals.is_empty() {
                        self.reveal_next();
                        markup.desired_scroll(ScrollMove::End);
                        return Action::Render;
                    }
                }
                KeyCode::Char('y' | 'n') => {
                    let success = key == KeyCode::Char('y');
                    db.schedule(id, success);
                    self.progress += 1;
                    self.next_card(db);
                    markup.desired_scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Right => {
                    if !self.due.is_empty() {
                        self.next_card(db);
                        self.due.insert(0, id);
                        markup.desired_scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                _ => {
                    if markup.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            },
            ReviewState::None | ReviewState::Done => {}
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.total = 0;
        self.progress = 0;
        self.state = ReviewState::None;
        self.reveals.clear();
        self.text.clear();
    }
}

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
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new().with_placeholder("content..."),
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
        menu: &mut Line,
        colors: &Colors,
        markup: &mut Markup,
        shortcuts: &mut Shortcuts,
    ) {
        let title = match self.state {
            CardEditorState::New => "New Card",
            CardEditorState::Edit(_) => "Edit Card",
        };
        menu.push_span(Span::raw(title));

        if self.preview {
            markup.render(self.editor.as_str(), area, buf, colors);
        } else {
            self.editor.render(area, buf, colors);
        }

        shortcuts.extend([Shortcut::new("Save", "^s"), Shortcut::new("Preview", "^p")]);
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        let ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let shift = modifiers.contains(KeyModifiers::SHIFT);

        if key == KeyCode::Char('p') && ctrl {
            self.preview = !self.preview;
            return Action::Render;
        }

        if self.preview {
            if markup.input(key, modifiers) {
                return Action::Render;
            }
        } else {
            match key {
                KeyCode::Up => {
                    if self.editor.move_cursor(CursorMove::Up, shift) {
                        return Action::Render;
                    }
                }
                KeyCode::Down => {
                    if self.editor.move_cursor(CursorMove::Down, shift) {
                        return Action::Render;
                    }
                }
                KeyCode::Char('s') => {
                    if ctrl {
                        markup.clear();
                        self.preview = false;
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
                    } else {
                        self.editor.push_char('s');
                    }
                    return Action::Render;
                }
                KeyCode::Char('p') => {
                    self.editor.push_char('p');
                    return Action::Render;
                }
                _ => {
                    if self.editor.input(key, modifiers) {
                        return Action::Render;
                    }
                }
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.preview = false;

        if let CardEditorState::Edit(_) = self.state {
            self.editor.clear();
        }
    }
}

pub struct CardsPage {
    cards: Vec<(CardId, CardStats)>,
    index: usize,
    state: CardState,
    sort: CardSort,
    search: TextInput,
    // todo: toggle deleted
}

struct CardStats {
    creation: UnixTime,
    difficulty: f32,
    score: MatchScore,
}

impl CardStats {
    const fn new(card: &Card, score: MatchScore) -> Self {
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

impl CardsPage {
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
            state: CardState::Browse,
            sort: CardSort::Newest,
            search: TextInput::new().with_placeholder("search..."),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.cards.extend(
            db.iter()
                .map(|(id, card)| (id, CardStats::new(card, MatchScore::default()))),
        );
        self.sort_cards();
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        menu: &mut Line,
        colors: &Colors,
        markup: &mut Markup,
        db: &Database,
        shortcuts: &mut Shortcuts,
    ) {
        menu.extend([
            Span::styled(
                format!("{} / {}", self.index + 1, self.cards.len()),
                STYLE_NONE.fg(colors.neutral),
            ),
            Span::raw("   "),
            Span::styled(
                match self.sort {
                    CardSort::Newest => "Newest",
                    CardSort::Oldest => "Oldest",
                    CardSort::Easy => "Easy",
                    CardSort::Hard => "Hard",
                    CardSort::Search => "Search",
                },
                STYLE_NONE.fg(colors.neutral),
            ),
        ]);

        match self.state {
            CardState::Browse => {
                if !self.search.is_empty() {
                    menu.extend([
                        Span::raw("   "),
                        Span::styled(
                            self.search.as_str().to_owned(),
                            STYLE_ITALIC.fg(colors.neutral),
                        ),
                    ]);
                }

                match self.cards.get(self.index) {
                    Some((id, _)) => {
                        let card = db.get(*id).unwrap();
                        markup.render(card.content.as_str(), area, buf, colors);

                        shortcuts.extend([
                            Shortcut::new("Browse", "⇄"),
                            Shortcut::new("Search", "/"),
                            Shortcut::new("Sort", "s"),
                        ]);
                        if !self.cards.is_empty() {
                            shortcuts.extend([
                                Shortcut::new("Edit", "e"),
                                Shortcut::new("Delete", "Del"),
                            ]);
                        }
                    }
                    None => {
                        Line::raw("no cards")
                            .alignment(Alignment::Center)
                            .render(area, buf);
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
                self.search.render(search_area, buf, colors);
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
                        markup.desired_scroll(ScrollMove::Start);
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
                        markup.desired_scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                KeyCode::Delete => {
                    if !self.cards.is_empty() {
                        let (id, _) = self.cards.remove(self.index);
                        db.remove(id);
                        if !self.cards.is_empty() {
                            self.index = self.index.min(self.cards.len() - 1);
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
                    markup.desired_scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Char('/') => {
                    self.state = CardState::Search;
                    return Action::Render;
                }
                _ => {
                    if markup.input(key, modifiers) {
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
        self.search.clear();
    }

    fn fetch_cards(&mut self, db: &Database, matcher: &mut Matcher) {
        if self.search.is_empty() {
            let all_cards = db
                .iter()
                .map(|(id, card)| (id, CardStats::new(card, MatchScore::default())));
            self.cards.extend(all_cards);
        } else {
            let matched_cards = db
                .iter()
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
}

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::prelude::*;

use crate::{
    app::{Action, Colors},
    database::*,
    editor::*,
    markup::*,
    settings::Settings,
    utils::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<CardId>),
    Cards,
    Settings,
}

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage,
    pub cards: CardsPage,
    pub settings: SettingsPage,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(),
            cards: CardsPage::new(),
            settings: SettingsPage::new(),
        }
    }
}

pub struct ReviewPage {
    due: Vec<CardId>,
    total: usize,
    progress: usize,
    desired_retention: f32,
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
            desired_retention: 0.0,
            state: ReviewState::None,
            reveals: Vec::new(),
            text: String::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database, desired_retention: f32) {
        self.due.extend(db.due().map(|(id, _)| id));
        self.total = self.due.len();
        self.desired_retention = desired_retention;

        if !self.due.is_empty() {
            self.next_card(db);
        }
    }

    fn next_card(&mut self, db: &Database) {
        self.reveals.clear();
        self.text.clear();

        if let Some(id) = self.due.pop() {
            let card_content = db.get(&id).unwrap().get_content();

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
        menu: &mut Menu,
        colors: &Colors,
        markup: &mut Markup,
        shortcuts: &mut Shortcuts,
    ) {
        match self.state {
            ReviewState::None => {
                Line::raw("no cards to review...")
                    .alignment(Alignment::Center)
                    .render(area.inner(MARGIN_CONTENT), buf);
            }
            ReviewState::Review(_) => {
                menu.push_span(Span::styled(
                    format!("{} / {}", self.progress, self.total),
                    STYLE_NONE.fg(colors.neutral),
                ));

                markup.render(&self.text, area, buf, colors);

                if !self.reveals.is_empty() {
                    shortcuts.extend([SHORTCUT_SHOW]);
                } else {
                    shortcuts.extend([SHORTCUT_YES, SHORTCUT_NO]);
                }
                if !self.due.is_empty() {
                    shortcuts.extend([SHORTCUT_SKIP]);
                }
                shortcuts.extend([SHORTCUT_EDIT, SHORTCUT_DELETE]);
            }
            ReviewState::Done => {
                Line::raw("done")
                    .alignment(Alignment::Center)
                    .render(area.inner(MARGIN_CONTENT), buf);
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        markup: &mut Markup,
        db: &mut Database,
    ) -> Action {
        match self.state {
            ReviewState::Review(id) => match key {
                KeyCode::Char('e') => return Action::Route(Route::Editor(Some(id))),
                KeyCode::Delete => {
                    db.remove(&id);
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
                    db.schedule(id, success, self.desired_retention);
                    self.progress += 1;
                    self.next_card(db);
                    markup.desired_scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Up => {
                    if markup.scroll(ScrollMove::Up(1)) {
                        return Action::Render;
                    }
                }
                KeyCode::Down => {
                    if markup.scroll(ScrollMove::Down(1)) {
                        return Action::Render;
                    }
                }
                KeyCode::Right => {
                    if !self.due.is_empty() {
                        self.next_card(db);
                        self.due.insert(0, id);
                        markup.desired_scroll(ScrollMove::Start);
                        return Action::Render;
                    }
                }
                _ => {}
            },
            ReviewState::None | ReviewState::Done => {}
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.total = 0;
        self.progress = 0;
        self.desired_retention = 0.0;
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
                let card = db.get(&id).unwrap();
                self.editor.clear();
                self.editor.push_str(card.get_content());
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
        menu: &mut Menu,
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

        shortcuts.extend([SHORTCUT_SAVE, SHORTCUT_PREVIEW]);
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

        match key {
            KeyCode::Up => {
                if self.preview {
                    if markup.scroll(ScrollMove::Up(1)) {
                        return Action::Render;
                    }
                } else {
                    if self.editor.move_cursor(CursorMove::Up, shift) {
                        return Action::Render;
                    }
                }
            }
            KeyCode::Down => {
                if self.preview {
                    if markup.scroll(ScrollMove::Down(1)) {
                        return Action::Render;
                    }
                } else {
                    if self.editor.move_cursor(CursorMove::Down, shift) {
                        return Action::Render;
                    }
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
                            self.editor.clear();
                            return Action::Render;
                        }
                        CardEditorState::Edit(id) => {
                            let card = db.get_mut(&id).unwrap();
                            card.set_content(self.editor.as_str());
                            self.editor.clear();
                            return Action::Route(Route::Review); // todo: go back?
                        }
                    }
                } else if !self.preview {
                    self.editor.push_char('s');
                    return Action::Render;
                }
            }
            KeyCode::Char('p') => {
                if ctrl {
                    self.preview = !self.preview;
                } else if !self.preview {
                    self.editor.push_char('p');
                }
                return Action::Render;
            }
            _ => {
                if !self.preview {
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
    cards: Vec<(CardId, MatchScore)>,
    index: usize,
    state: CardState,
    sort: CardSort,
    search: TextInput,
}

enum CardState {
    Browse,
    Search,
}

enum CardSort {
    Newest,
    Oldest,
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

    fn fetch_cards(&mut self, db: &mut Database) {
        if self.search.is_empty() {
            let all_cards = db.iter().map(|(id, _)| (*id, MatchScore::default()));
            self.cards.extend(all_cards);
        } else {
            let matched_cards = db
                .search(self.search.as_str())
                .map(|(id, _, score)| (id, score));
            self.cards.extend(matched_cards);
        }
    }

    fn sort_cards(&mut self) {
        match self.sort {
            CardSort::Newest => {
                self.cards
                    .sort_unstable_by_key(|(id, _)| std::cmp::Reverse(*id));
            }
            CardSort::Oldest => {
                self.cards.sort_unstable_by_key(|(id, _)| *id);
            }
            CardSort::Search => {
                self.cards
                    .sort_by_key(|(_, score)| std::cmp::Reverse(*score));
            }
        }
    }

    pub fn on_enter(&mut self, db: &mut Database) {
        self.fetch_cards(db);
        self.sort_cards();
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        menu: &mut Menu,
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
                        let card = db.get(id).unwrap();
                        markup.render(card.get_content(), area, buf, colors);

                        shortcuts.extend([SHORTCUT_BROWSE, SHORTCUT_SEARCH, SHORTCUT_SORT]);
                        if !self.cards.is_empty() {
                            shortcuts.extend([SHORTCUT_EDIT, SHORTCUT_DELETE]);
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
                shortcuts.extend([SHORTCUT_CONFIRM]);
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
                KeyCode::Up => {
                    if markup.scroll(ScrollMove::Up(1)) {
                        return Action::Render;
                    }
                }
                KeyCode::Down => {
                    if markup.scroll(ScrollMove::Down(1)) {
                        return Action::Render;
                    }
                }
                KeyCode::Delete => {
                    if !self.cards.is_empty() {
                        let (id, _) = self.cards.remove(self.index);
                        db.remove(&id);
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
                    if self.search.is_empty() {
                        match self.sort {
                            CardSort::Newest => self.sort = CardSort::Oldest,
                            CardSort::Oldest => self.sort = CardSort::Newest,
                            CardSort::Search => todo!(),
                        }
                    } else {
                        match self.sort {
                            CardSort::Newest => self.sort = CardSort::Oldest,
                            CardSort::Oldest => self.sort = CardSort::Search,
                            CardSort::Search => self.sort = CardSort::Newest,
                        }
                    }
                    self.index = 0;
                    self.sort_cards();
                    markup.desired_scroll(ScrollMove::Start);
                    return Action::Render;
                }
                KeyCode::Char('/') => {
                    self.state = CardState::Search;
                    return Action::Render;
                }
                _ => {}
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
                    self.fetch_cards(db);
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
}

pub struct SettingsPage {
    state: SettingsState,
    text: String,
}

enum SettingsState {
    Database,
    Retention,
}

impl SettingsState {
    fn next(&mut self) {
        *self = match self {
            Self::Database => Self::Retention,
            Self::Retention => Self::Database,
        };
    }

    fn prev(&mut self) {
        *self = match self {
            Self::Database => Self::Retention,
            Self::Retention => Self::Database,
        };
    }
}

impl SettingsPage {
    pub const fn new() -> Self {
        Self {
            state: SettingsState::Database,
            text: String::new(),
        }
    }

    fn update_text(&mut self, settings: &Settings) {
        self.text.clear();

        match self.state {
            SettingsState::Database => {
                self.text.extend([
                    "The path for your database file.\n\n|_",
                    &settings.database.to_string_lossy(),
                    "_\n\nThis is where all your cards are stored. ",
                    "You can move the file by setting a new value, ",
                    "or open another database file.",
                ]);
            }
            SettingsState::Retention => {
                self.text.extend([
                    "The desired retention for your cards.\n\n|*",
                    &format!("{:.0}", settings.desired_retention * 100.0),
                    "%*\n\nA higher retention leads to shorter intervals and more reviews per day. ",
                    "The default value is 80%."
                ]);
            }
        }
    }

    pub fn on_enter(&mut self, settings: &Settings) {
        self.update_text(settings);
    }

    pub fn on_render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        menu: &mut Menu,
        colors: &Colors,
        markup: &mut Markup,
        shortcuts: &mut Shortcuts,
    ) {
        shortcuts.push(SHORTCUT_BROWSE_HORIZONTAL);

        match &self.state {
            SettingsState::Database => {
                menu.push_span(Span::raw("Database"));
                markup.render(&self.text, area, buf, colors);
            }
            SettingsState::Retention => {
                menu.push_span(Span::raw("Desired Retention"));
                markup.render(&self.text, area, buf, colors);
                shortcuts.push(SHORTCUT_ADJUST);
            }
        }
    }

    pub fn on_input(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        settings: &mut Settings,
    ) -> Action {
        match &self.state {
            SettingsState::Database => match key {
                KeyCode::Right => {
                    self.state.next();
                    self.update_text(settings);
                    Action::Render
                }
                KeyCode::Left => {
                    self.state.prev();
                    self.update_text(settings);
                    Action::Render
                }
                _ => Action::None,
            },
            SettingsState::Retention => match key {
                KeyCode::Right => {
                    self.state.next();
                    self.update_text(settings);
                    Action::Render
                }
                KeyCode::Left => {
                    self.state.prev();
                    self.update_text(settings);
                    Action::Render
                }
                KeyCode::Up => {
                    settings.desired_retention += 0.01; // todo: clamp, also maybe u8?
                    self.update_text(settings);
                    Action::Render
                }
                KeyCode::Down => {
                    settings.desired_retention -= 0.01; // todo: clamp, also maybe u8?
                    self.update_text(settings);
                    Action::Render
                }
                _ => Action::None,
            },
        }
    }

    pub fn on_exit(&mut self) {}
}

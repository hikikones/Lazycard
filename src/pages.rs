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
    pub review: Review,
    pub editor: CardEditor,
    pub cards: Cards,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: Review::new(),
            editor: CardEditor::new(),
            cards: Cards::new(),
        }
    }
}

pub struct Review {
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

impl Review {
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
        self.due.extend(db.iter().rev().map(|(id, _)| id));
        self.total = self.due.len();

        if !self.due.is_empty() {
            self.next_card(db);
        }
    }

    fn next_card(&mut self, db: &Database) {
        self.reveals.clear();
        self.text.clear();

        if let Some(id) = self.due.pop() {
            let card_content = db.get(&id).unwrap().0.as_str();

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
        mut area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        markup: &mut Markup,
    ) {
        match self.state {
            ReviewState::None => {
                Line::raw("no cards to review...")
                    .alignment(Alignment::Center)
                    .render(area.inner(MARGIN_CONTENT), buf);
            }
            ReviewState::Review(_) => {
                area.y += 1;
                area.height -= 1;

                let mut progress_line = Line::default().alignment(Alignment::Center);
                progress_line.push_span(Span::styled(
                    format!("{} / {}", self.progress, self.total),
                    STYLE_NONE.fg(colors.neutral),
                ));
                progress_line.render(area, buf);

                markup.render(&self.text, area.inner(MARGIN_CONTENT), buf, colors);
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
            ReviewState::Review(id) => {
                match key {
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
                    KeyCode::Up => {
                        // todo: successful recall
                        // fixme: activates when scrolling with touchpad?
                        if markup.scroll(ScrollMove::Up(1)) {
                            return Action::Render;
                        }
                    }
                    KeyCode::Down => {
                        // todo: unsuccessful recall
                        // fixme: activates when scrolling with touchpad?
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
                }
            }
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

    pub fn shortcuts(&self, shortcuts: &mut Shortcuts) {
        match self.state {
            ReviewState::Review(_) => {
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
            ReviewState::None | ReviewState::Done => {}
        }
    }
}

pub struct CardEditor {
    editor: TextEditor,
    state: CardEditorState,
    preview: bool,
}

enum CardEditorState {
    New,
    Edit(CardId),
}

impl CardEditor {
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
                self.editor.push_str(card.0.as_str());
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
        mut area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        markup: &mut Markup,
    ) {
        let title = match self.state {
            CardEditorState::New => "New Card",
            CardEditorState::Edit(_) => "Edit Card",
        };

        area.y += 1;
        area.height -= 1;

        Line::raw(title)
            .alignment(Alignment::Center)
            .render(area, buf);

        if self.preview {
            markup.render(
                self.editor.as_str(),
                area.inner(MARGIN_CONTENT),
                buf,
                colors,
            );
        } else {
            self.editor.render(area.inner(MARGIN_CONTENT), buf, colors);
        }
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
                            card.0 = self.editor.as_str().to_owned();
                            self.editor.clear();
                            return Action::Route(Route::Review);
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

    pub fn shortcuts(&self, shortcuts: &mut Shortcuts) {
        shortcuts.extend([SHORTCUT_SAVE, SHORTCUT_PREVIEW]);
    }
}

pub struct Cards {
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

impl Cards {
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
            let all_cards = db.iter().map(|(id, _)| (*id, MatchScore(0)));
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
                    .sort_unstable_by_key(|(id, _)| std::cmp::Reverse(id.0));
            }
            CardSort::Oldest => {
                self.cards.sort_unstable_by_key(|(id, _)| id.0);
            }
            CardSort::Search => {
                self.cards
                    .sort_by_key(|(_, score)| std::cmp::Reverse(score.0));
            }
        }
    }

    pub fn on_enter(&mut self, db: &mut Database) {
        self.fetch_cards(db);
        self.sort_cards();
    }

    pub fn on_render(
        &mut self,
        mut area: Rect,
        buf: &mut Buffer,
        colors: &Colors,
        markup: &mut Markup,
        db: &Database,
    ) {
        area.y += 1;
        area.height -= 1;

        let mut menu = Line::default().alignment(Alignment::Center);
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
        menu.render(area, buf);

        if let CardState::Search = self.state {
            area.y += 2;
            area.height -= 2;

            let search_area = Rect {
                height: 1,
                width: area.width / 2,
                x: area.x + area.width / 4,
                y: area.y,
            };
            self.search.render(search_area, buf, colors);
            return;
        }

        if !self.search.is_empty() {
            area.y += 2;
            area.height -= 2;

            let mut bar = Line::default().alignment(Alignment::Center);
            bar.push_span(Span::styled(
                self.search.as_str(),
                STYLE_ITALIC.fg(colors.neutral),
            ));
            bar.render(Rect { height: 1, ..area }, buf);
        }

        let area = area.inner(MARGIN_CONTENT);

        let Some((id, _)) = self.cards.get(self.index) else {
            Line::raw("no cards")
                .alignment(Alignment::Center)
                .render(area, buf);
            return;
        };

        let Some(card) = db.get(id) else {
            Line::raw("todo: card not found in database")
                .alignment(Alignment::Center)
                .render(area, buf);
            return;
        };

        markup.render(&card.0, area, buf, colors);
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

    pub fn shortcuts(&self, shortcuts: &mut Shortcuts) {
        match self.state {
            CardState::Browse => {
                shortcuts.extend([SHORTCUT_BROWSE, SHORTCUT_SEARCH, SHORTCUT_SORT]);
                if !self.cards.is_empty() {
                    shortcuts.extend([SHORTCUT_EDIT, SHORTCUT_DELETE]);
                }
            }
            CardState::Search => {
                shortcuts.extend([SHORTCUT_CONFIRM]);
            }
        }
    }
}

use std::sync::LazyLock;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use tui_textarea::TextArea;

use crate::{app::Action, database::*, markup::*, utils::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard,
}

impl Route {
    pub const fn title(self) -> &'static str {
        match self {
            Route::Review => "Review",
            Route::AddCard => "Add Card",
            Route::EditCard => "Edit Card",
        }
    }
}

pub struct Pages {
    pub review: Review,
    pub add_card: AddCard,
    pub edit_card: EditCard,
}

impl Pages {
    pub fn new() -> Self {
        Self {
            review: Review::new(),
            add_card: AddCard::new(),
            edit_card: EditCard::new(),
        }
    }
}

pub struct Review {
    due: Vec<CardId>,
    index: usize,
    text: String,
}

impl Review {
    pub const fn new() -> Self {
        Self {
            due: Vec::new(),
            index: 0,
            text: String::new(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        self.due.extend(db.iter().map(|(id, _)| id));
        if let Some(id) = self.due.get(self.index) {
            if let Some(card) = db.get(id) {
                self.text.push_str(card.0.as_str());
            }
        }
    }

    pub fn on_render(&self, mut area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        for block in BlockParser::new(&self.text) {
            match block {
                BlockElement::Paragraph { alignment, text } => {
                    let ansi_markup = InlineParser::new(text).into_ansi().replace('\n', " ");
                    let mut ansi_parser = AnsiParser::new("");
                    for wrapped_line in textwrap::fill(&ansi_markup, width).lines() {
                        let mut line = Line::default();
                        ansi_parser.continue_with(wrapped_line);
                        for (tag, span) in &mut ansi_parser {
                            let style = match tag {
                                AnsiTag::Text => Style::new(),
                                AnsiTag::Bold => Style::new().bold(),
                                AnsiTag::Italic => Style::new().italic(),
                            };
                            line.push_span(Span::styled(span, style));
                        }
                        line.alignment(alignment).render(area, buf);
                        area.y += 1;
                    }
                }
                BlockElement::Code { language, text } => {
                    static SYNTAX_SET: LazyLock<SyntaxSet> =
                        LazyLock::new(|| SyntaxSet::load_defaults_newlines());
                    static THEME_SET: LazyLock<ThemeSet> =
                        LazyLock::new(|| ThemeSet::load_defaults());

                    let syntax = if language.is_empty() {
                        SYNTAX_SET.find_syntax_plain_text()
                    } else {
                        SYNTAX_SET
                            .find_syntax_by_token(language)
                            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
                    };
                    let mut highlighter =
                        HighlightLines::new(syntax, &THEME_SET.themes["base16-eighties.dark"]);

                    for code_line in LinesWithEndings::from(text) {
                        match highlighter.highlight_line(code_line, &SYNTAX_SET) {
                            Ok(spans) => {
                                let mut line: Line<'_> = Line::default();
                                for (style, span) in spans {
                                    let mut modifiers = Modifier::empty();
                                    if style.font_style.contains(FontStyle::BOLD) {
                                        modifiers.insert(Modifier::BOLD);
                                    }
                                    if style.font_style.contains(FontStyle::ITALIC) {
                                        modifiers.insert(Modifier::ITALIC);
                                    }
                                    if style.font_style.contains(FontStyle::UNDERLINE) {
                                        modifiers.insert(Modifier::UNDERLINED);
                                    }
                                    let fg = Color::Rgb(
                                        style.foreground.r,
                                        style.foreground.g,
                                        style.foreground.b,
                                    );
                                    line.push_span(Span::styled(
                                        span,
                                        Style::new().add_modifier(modifiers).fg(fg),
                                    ));
                                }
                                line.render(area, buf);
                                area.y += 1;
                            }
                            Err(_) => {
                                Line::raw(code_line).render(area, buf);
                                area.y += 1;
                            }
                        }
                    }
                }
            }
            area.y += 1;
        }
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::AddCard),
                KeyCode::Char('e') => return Action::Route(Route::EditCard),
                KeyCode::Delete => {
                    // todo: delete card
                }
                KeyCode::Char(' ') => {
                    // todo: show answer
                }
                KeyCode::Up => {
                    // todo: successful recall
                    // fixme: activates when scrolling with touchpad?
                }
                KeyCode::Down => {
                    // todo: unsuccessful recall
                    // fixme: activates when scrolling with touchpad?
                }
                KeyCode::Right => {
                    self.index = (self.index + 1) % self.due.len();
                    if let Some(id) = self.due.get(self.index) {
                        if let Some(card) = db.get(id) {
                            self.text.clear();
                            self.text.push_str(card.0.as_str());
                            return Action::Render;
                        }
                    }
                }
                _ => {}
            }
        }

        Action::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.index = 0;
        self.text.clear();
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[SHORTCUT_EDIT, SHORTCUT_DELETE, SHORTCUT_MENU, SHORTCUT_QUIT]
    }
}

pub struct AddCard {
    editor: TextArea<'static>,
}

impl AddCard {
    pub fn new() -> Self {
        Self {
            editor: TextArea::default(),
        }
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo
    }

    pub fn on_render(&self, area: Rect, buf: &mut Buffer) {
        self.editor.render(area, buf);
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Tab => return Action::Route(Route::Review),
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        let content = self.editor.lines().join("\n");
                        self.editor = TextArea::default();
                        db.add(Card::new(content));
                        return Action::Render;
                    } else {
                        self.editor.input(key);
                        return Action::Render;
                    }
                }
                KeyCode::Char('p') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // todo: toggle preview
                    } else {
                        self.editor.input(key);
                        return Action::Render;
                    }
                }
                _ => {
                    self.editor.input(key);
                    return Action::Render;
                }
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, s: String) -> Action {
        //todo
        Action::None
    }

    pub fn on_exit(&mut self) {
        //todo
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[
            SHORTCUT_SAVE,
            SHORTCUT_PREVIEW,
            SHORTCUT_MENU,
            SHORTCUT_QUIT,
        ]
    }
}

pub struct EditCard;

impl EditCard {
    pub const fn new() -> Self {
        Self
    }

    pub fn on_enter(&mut self, db: &Database) {
        //todo
    }

    pub fn on_render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("todo: edit card...").render(area, buf);
    }

    pub fn on_input(&mut self, key: KeyEvent, db: &mut Database) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Esc => return Action::Quit,
                KeyCode::Char('s') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // todo: save and go back
                        return Action::Route(Route::Review);
                    }
                }
                KeyCode::Char('c') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        // todo: cancel and go back
                        return Action::Route(Route::Review);
                    }
                }
                _ => {}
            }
        }

        Action::None
    }

    pub fn on_paste(&mut self, s: String) -> Action {
        //todo
        Action::None
    }

    pub fn on_exit(&mut self) {
        //todo
    }

    pub fn shortcuts<'a>(&'a self) -> &'a [Shortcut] {
        &[SHORTCUT_SAVE, SHORTCUT_CANCEL, SHORTCUT_QUIT]
    }
}

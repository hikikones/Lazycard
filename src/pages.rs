use std::sync::LazyLock;

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use tui_textarea::TextArea;

use crate::{
    app::{Areas, InputEvent, Message},
    database::*,
    markup::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    AddCard,
    EditCard,
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

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Review")
            .centered()
            .render(areas.header, buf);

        let mut body = areas.body;
        let width = body.width;

        for block in BlockParser::new(&self.text) {
            match block {
                BlockElement::Paragraph { alignment, text } => {
                    let wraps = textwrap::wrap(text, textwrap::Options::new(width as usize));
                    let mut inline_parser = InlineParser::new("");
                    for wrapped_line in wraps.iter() {
                        let mut line = Line::default();
                        inline_parser.continue_with(&wrapped_line);
                        for element in &mut inline_parser {
                            const NONE: Style = Style::new();
                            const BOLD: Style = NONE.add_modifier(Modifier::BOLD);
                            const CURSIVE: Style = NONE.add_modifier(Modifier::ITALIC);
                            let (span, style) = match element {
                                InlineElement::Text(s) => (s, NONE),
                                InlineElement::Bold(s) => (s, BOLD),
                                InlineElement::Cursive(s) => (s, CURSIVE),
                            };
                            line.push_span(Span::styled(span, style));
                        }
                        line.alignment(alignment).render(body, buf);
                        body.y += 1;
                    }
                    body.y += 1;
                }
                BlockElement::Code { language, text } => {
                    static SYNTAX_SET: LazyLock<SyntaxSet> =
                        LazyLock::new(|| SyntaxSet::load_defaults_newlines());
                    static THEME_SET: LazyLock<ThemeSet> =
                        LazyLock::new(|| ThemeSet::load_defaults());

                    let syntax = SYNTAX_SET
                        .find_syntax_by_extension(language)
                        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
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
                                line.render(body, buf);
                                body.y += 1;
                            }
                            Err(_) => {
                                Line::raw(code_line).render(body, buf);
                                body.y += 1;
                            }
                        }
                    }
                }
            }
        }

        Line::from(
            "[esc]Quit  [tab]Menu  [e]Edit  [del]Delete  [space]Show  [↑]Yes  [↓]No  [→]Next",
        )
        .centered()
        .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::AddCard),
                    KeyCode::Char('e') => return Message::Route(Route::EditCard),
                    KeyCode::Delete => todo!("delete"),
                    KeyCode::Char(' ') => todo!("show answer"),
                    KeyCode::Up => todo!("yes"), // fixme: activates when scrolling with touchpad?
                    KeyCode::Down => todo!("no"), // fixme: activates when scrolling with touchpad?
                    KeyCode::Right => {
                        self.index = (self.index + 1) % self.due.len();
                        if let Some(id) = self.due.get(self.index) {
                            if let Some(card) = db.get(id) {
                                self.text.clear();
                                self.text.push_str(card.0.as_str());
                                return Message::Render;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Message::None
    }

    pub fn on_exit(&mut self) {
        self.due.clear();
        self.index = 0;
        self.text.clear();
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

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Add Card")
            .centered()
            .render(areas.header, buf);

        self.editor.render(areas.body, buf);

        Line::from("[esc]Quit  [tab]Menu  [ctrl-s]Save  [ctrl-p]Preview")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Tab => return Message::Route(Route::Review),
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            let content = self.editor.lines().join("\n");
                            self.editor = TextArea::default();
                            db.add(Card::new(content));
                            return Message::Render;
                        } else {
                            self.editor.input(key);
                            return Message::Render;
                        }
                    }
                    KeyCode::Char('p') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            todo!("preview");
                        } else {
                            self.editor.input(key);
                            return Message::Render;
                        }
                    }
                    _ => {
                        self.editor.input(key);
                        return Message::Render;
                    }
                }
            }
        }

        Message::None
    }

    pub fn on_exit(&mut self) {
        //todo
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

    pub fn on_render(&self, areas: &Areas, buf: &mut Buffer) {
        Line::from("Lazycard — Edit Card")
            .centered()
            .render(areas.header, buf);

        Paragraph::new("edit card...").render(areas.body, buf);

        Line::from("[esc]Quit  [ctrl-s]Save  [ctrl-c]Cancel")
            .centered()
            .render(areas.footer, buf);
    }

    pub fn on_input(&mut self, event: InputEvent, db: &mut Database) -> Message {
        if let InputEvent::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Message::Quit,
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo: save and go back
                            return Message::Route(Route::Review);
                        }
                    }
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // todo: cancel and go back
                            return Message::Route(Route::Review);
                        }
                    }
                    _ => {}
                }
            }
        }

        Message::None
    }

    pub fn on_exit(&mut self) {
        //todo
    }
}

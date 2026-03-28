use database::*;
use ratatui::{
    CompletedFrame,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};
use widgets::{Markup, Shortcut, Shortcuts};

use crate::{pages::*, settings::Settings, symbols, terminal::Terminal};

pub struct App {
    route: Route,
    pages: Pages,
    database: Database,
    settings: Settings,
    markup: Markup,
    matcher: Matcher,
    title_line: Line<'static>,
    nav_line: Line<'static>,
    menu_line: Line<'static>,
    shortcuts: Shortcuts,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Quit,
}

impl App {
    pub fn new(database: Database, external_editor: bool) -> Self {
        let settings = Settings::default();

        let mut title_line = Line::default().centered();
        title_line.push_span(Span::styled("lazycard", settings.neutral()));

        let markup = Markup::new(settings.syntax_highlighting());
        let shortcuts = Shortcuts::new().with_colors(Color::Reset, settings.primary());

        Self {
            route: Route::Review,
            pages: Pages::new(external_editor, settings.colors()),
            database,
            settings,
            markup,
            matcher: Matcher::new(),
            title_line,
            nav_line: Line::default().centered(),
            menu_line: Line::default().centered(),
            shortcuts,
        }
    }

    pub fn run(&mut self, mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
        self.pages.review.on_enter(&self.database);
        self.render(&mut terminal)?;

        loop {
            let action = match ratatui::crossterm::event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Esc => Action::Quit,
                            KeyCode::Tab => match self.route {
                                Route::Review => Action::Route(Route::Editor(None)),
                                Route::Editor(_) => Action::Route(Route::Cards),
                                Route::Cards => Action::Route(Route::Review),
                            },
                            KeyCode::BackTab => match self.route {
                                Route::Review => Action::Route(Route::Cards),
                                Route::Editor(_) => Action::Route(Route::Review),
                                Route::Cards => Action::Route(Route::Editor(None)),
                            },
                            _ => match self.route {
                                Route::Review => self.pages.review.on_input(
                                    key.code,
                                    key.modifiers,
                                    &mut self.markup,
                                    &mut self.database,
                                ),
                                Route::Editor(_) => self.pages.editor.on_input(
                                    key.code,
                                    key.modifiers,
                                    &mut self.markup,
                                    &mut self.database,
                                    &mut terminal,
                                )?,
                                Route::Cards => self.pages.cards.on_input(
                                    key.code,
                                    key.modifiers,
                                    &mut self.markup,
                                    &mut self.database,
                                    &mut self.matcher,
                                ),
                            },
                        }
                    } else {
                        Action::None
                    }
                }
                Event::Resize(_, _) => Action::Render,
                _ => Action::None,
            };

            match action {
                Action::None => {}
                Action::Render => {
                    self.render(&mut terminal)?;
                }
                Action::Route(route) => {
                    match self.route {
                        Route::Review => self.pages.review.on_exit(),
                        Route::Editor(_) => self.pages.editor.on_exit(),
                        Route::Cards => self.pages.cards.on_exit(),
                    }

                    self.route = route;
                    self.markup.clear();

                    match route {
                        Route::Review => self.pages.review.on_enter(&self.database),
                        Route::Editor(id) => {
                            self.pages
                                .editor
                                .on_enter(id, &self.database, &mut terminal)?
                        }
                        Route::Cards => self.pages.cards.on_enter(&mut self.database),
                    }

                    self.render(&mut terminal)?;
                }
                Action::Quit => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn quit(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.database.save()
    }

    fn render<'a>(&'a mut self, terminal: &'a mut Terminal) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let colors = self.settings.colors();

            let [
                title_area,
                _,
                nav_area,
                menu_area,
                body_area,
                shortcuts_page_area,
                shortcuts_app_area,
            ] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

            // Title
            (&self.title_line).render(title_area, buf);

            // Navigation
            for route in [Route::Review, Route::Editor(None), Route::Cards] {
                let (name, is_current) = match route {
                    Route::Review => ("Review", matches!(self.route, Route::Review)),
                    Route::Editor(_) => ("Editor", matches!(self.route, Route::Editor(_))),
                    Route::Cards => ("Cards", matches!(self.route, Route::Cards)),
                };
                let style = if is_current {
                    Style::new().bold().fg(self.settings.primary())
                } else {
                    Style::new()
                };
                self.nav_line
                    .extend([Span::styled(name, style), Span::raw("   ")]);
            }
            self.nav_line.spans.pop();
            (&self.nav_line).render(nav_area, buf);
            self.nav_line.spans.clear();

            // Body
            const MAX_WIDTH: u16 = 64;
            const MARGIN: u16 = 2;
            let body = center_horizontal(body_area, Constraint::Length(MAX_WIDTH + MARGIN))
                .inner(Margin::new(MARGIN, MARGIN));
            match self.route {
                Route::Review => {
                    self.pages.review.on_render(
                        body,
                        buf,
                        colors,
                        &mut self.menu_line,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(
                        body,
                        buf,
                        colors,
                        &mut self.menu_line,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Cards => {
                    self.pages.cards.on_render(
                        body,
                        buf,
                        &self.database,
                        colors,
                        &mut self.menu_line,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
            }

            // Menu
            (&self.menu_line).render(menu_area, buf);
            self.menu_line.spans.clear();

            // Shortcuts
            self.shortcuts.render(shortcuts_page_area, buf);
            self.shortcuts.clear();

            self.shortcuts.extend([
                Shortcut::new("Quit", symbols::ESCAPE),
                Shortcut::new("Navigate", symbols::shift!(symbols::TAB)),
            ]);
            self.shortcuts.render(shortcuts_app_area, buf);
            self.shortcuts.clear();
        })
    }
}

pub struct Matcher {
    matcher: nucleo_matcher::Matcher,
    pattern: nucleo_matcher::pattern::Pattern,
    buffer: Vec<char>,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
            pattern: nucleo_matcher::pattern::Pattern::new(
                "",
                nucleo_matcher::pattern::CaseMatching::Smart,
                nucleo_matcher::pattern::Normalization::Smart,
                nucleo_matcher::pattern::AtomKind::Fuzzy,
            ),
            buffer: Vec::new(),
        }
    }

    pub fn update(&mut self, pattern: &str) {
        self.pattern.reparse(
            pattern,
            nucleo_matcher::pattern::CaseMatching::Smart,
            nucleo_matcher::pattern::Normalization::Smart,
        );
    }

    pub fn score(&mut self, haystack: &str) -> Option<u32> {
        self.pattern.score(
            nucleo_matcher::Utf32Str::new(haystack, &mut self.buffer),
            &mut self.matcher,
        )
    }
}

fn center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub trait CardsIterExt<'a> {
    fn due(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn active(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn search(
        self,
        pattern: &str,
        matcher: &'a mut Matcher,
    ) -> impl Iterator<Item = (CardId, &'a Card, u32)>;
}

impl<'a, I> CardsIterExt<'a> for I
where
    I: Iterator<Item = (CardId, &'a Card)>,
{
    fn due(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        let now = UnixTime::now();
        self.filter(move |(_, card)| !card.archived && card.is_due(now))
    }

    fn active(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        self.filter(|(_, card)| !card.archived)
    }

    fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        self.filter(|(_, card)| card.archived)
    }

    fn search(
        self,
        pattern: &str,
        matcher: &'a mut Matcher,
    ) -> impl Iterator<Item = (CardId, &'a Card, u32)> {
        matcher.update(pattern);
        self.filter_map(|(id, card)| {
            matcher
                .score(card.content.as_str())
                .map(|score| (id, card, score))
        })
    }
}

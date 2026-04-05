use std::path::PathBuf;

use database::{Card, CardId, Database, UnixTime};
use ratatui::{
    CompletedFrame,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Style},
};
use widgets::{Markup, Shortcut, Shortcuts, TextSegment};

use crate::{pages::*, settings::Settings, symbols, terminal::Terminal};

pub struct App {
    route: Route,
    state: State,
    pages: Pages,
    database: Database,
    settings: Settings,
    markup: Markup,
    matcher: Matcher,
    text: TextSegment,
    shortcuts: Shortcuts,
}

enum State {
    Route,
    Logs,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Log(Log),
    Quit,
}

impl App {
    pub fn new(database: Database, settings_path: Option<PathBuf>) -> Self {
        let mut logs = LogsPage::new();
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message"));
        logs.enqueue(Log::new("message dlkaj waj ioajioawjd ioajdioajd ioajdioajdioajdioajidoajdioajdioajdioajdioajwidoajiodjaidojaiodjwiodjaioj"));

        let settings = Settings::read(settings_path.clone())
            .inspect_err(|err| logs.enqueue(Log::new(err)))
            .unwrap_or_default()
            .with_path(settings_path);
        let markup = Markup::new(settings.syntax_highlighting());
        let shortcuts = Shortcuts::new().with_colors(Color::Reset, settings.primary());

        let colors = settings.colors();
        let pages = Pages {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(colors),
            cards: CardsPage::new(colors),
            logs,
        };

        Self {
            route: Route::Review,
            state: State::Route,
            pages,
            database,
            settings,
            markup,
            matcher: Matcher::new(),
            text: TextSegment::new().with_alignment(Alignment::Center),
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
                            KeyCode::Tab | KeyCode::BackTab => match self.state {
                                State::Route => {
                                    let next_route = if key.code == KeyCode::Tab {
                                        self.route.next()
                                    } else {
                                        self.route.prev()
                                    };
                                    Action::Route(next_route)
                                }
                                State::Logs => {
                                    self.state = State::Route;
                                    self.pages.logs.on_exit();
                                    Action::Render
                                }
                            },
                            KeyCode::Char('l') => {
                                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                                if ctrl && !self.pages.logs.is_empty() {
                                    match self.state {
                                        State::Route => {
                                            self.state = State::Logs;
                                            self.pages.logs.on_enter();
                                        }
                                        State::Logs => {
                                            self.state = State::Route;
                                            self.pages.logs.on_exit();
                                        }
                                    }
                                    Action::Render
                                } else {
                                    self.on_input(key, &mut terminal)
                                }
                            }
                            _ => self.on_input(key, &mut terminal),
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
                        Route::Editor(id) => self.pages.editor.on_enter(id, &self.database),
                        Route::Cards => self.pages.cards.on_enter(&mut self.database),
                    }

                    self.render(&mut terminal)?;
                }
                Action::Log(log) => {
                    self.pages.logs.enqueue(log);
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

            // Layout
            let [
                nav_area,
                menu_area,
                body_area,
                shortcuts_page_area,
                shortcuts_app_area,
            ] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

            // Navigation
            const SPACING: &str = "   ";
            for (route, name, spacing) in [
                (Route::Review, "Review", SPACING),
                (Route::Editor(None), "Editor", SPACING),
                (Route::Cards, "Cards", ""),
            ] {
                let is_current =
                    std::mem::discriminant(&route) == std::mem::discriminant(&self.route);
                let style = if is_current {
                    Style::new().fg(colors.primary).bold()
                } else {
                    Style::new()
                };
                self.text.extend([(name, style), (spacing, Style::new())]);
            }
            self.text.render(nav_area, buf);
            self.text.clear();

            // Body
            const MAX_WIDTH: u16 = 64;
            const MARGIN: u16 = 1;
            let body = center_horizontal(body_area, Constraint::Length(MAX_WIDTH + MARGIN))
                .inner(Margin::new(MARGIN, MARGIN));
            match self.state {
                State::Route => match self.route {
                    Route::Review => {
                        self.pages.review.on_render(
                            body,
                            buf,
                            colors,
                            &mut self.text,
                            &mut self.markup,
                            &mut self.shortcuts,
                        );
                    }
                    Route::Editor(_) => {
                        self.pages.editor.on_render(
                            body,
                            buf,
                            colors,
                            &mut self.text,
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
                            &mut self.text,
                            &mut self.markup,
                            &mut self.shortcuts,
                        );
                    }
                },
                State::Logs => {
                    self.pages.logs.on_render(
                        body,
                        buf,
                        colors,
                        &mut self.text,
                        &mut self.shortcuts,
                    );
                }
            }

            // Menu
            self.text.render(menu_area, buf);
            self.text.clear();

            // Page shortcuts
            self.shortcuts.render(shortcuts_page_area, buf);
            self.shortcuts.clear();

            // App shortcuts
            self.shortcuts.extend([
                Shortcut::new("Quit", symbols::ESCAPE),
                Shortcut::new("Navigate", symbols::shift!(symbols::TAB)),
            ]);

            if !self.pages.logs.is_empty() {
                let key = symbols::ctrl!("l");
                let new_logs = self.pages.logs.queue_len();
                if new_logs > 0 {
                    utils::format_int(new_logs, |new_logs| {
                        self.shortcuts.push_iter(["Logs(", new_logs, ")"], key);
                    });
                } else {
                    self.shortcuts.push(Shortcut::new("Logs", key));
                }
            }

            self.shortcuts.render(shortcuts_app_area, buf);
            self.shortcuts.clear();
        })
    }

    fn on_input(&mut self, key: KeyEvent, terminal: &mut Terminal) -> Action {
        match self.state {
            State::Route => match self.route {
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
                    terminal,
                ),
                Route::Cards => self.pages.cards.on_input(
                    key.code,
                    key.modifiers,
                    &mut self.markup,
                    &mut self.database,
                    &mut self.matcher,
                ),
            },
            State::Logs => match self.pages.logs.on_input(key.code, key.modifiers) {
                LogsAction::None => Action::None,
                LogsAction::Render => Action::Render,
                LogsAction::Done => {
                    self.state = State::Route;
                    self.pages.logs.on_exit();
                    Action::Render
                }
            },
        }
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

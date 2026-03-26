use crossterm::event::{Event, KeyCode, KeyEventKind};
use layout::Flex;
use ratatui::{prelude::*, CompletedFrame};

use crate::{database::*, markup::Markup, pages::*, terminal::Terminal};

pub struct App {
    route: Route,
    pages: Pages,
    db: Database,
    colors: Colors,
    markup: Markup,
    matcher: Matcher,
    title_line: Line<'static>,
    nav_line: Line<'static>,
    menu_line: Line<'static>,
    shortcuts: Shortcuts<'static>,
}

pub struct Colors {
    pub accent: Color,
    pub on_accent: Color,
    pub neutral: Color,
    pub syntax_highlighting: &'static str,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Quit,
}

impl App {
    pub fn new(database: Database, external_editor: bool) -> Self {
        let colors =
            match terminal_colorsaurus::theme_mode(terminal_colorsaurus::QueryOptions::default())
                .unwrap_or(terminal_colorsaurus::ThemeMode::Dark)
            {
                terminal_colorsaurus::ThemeMode::Dark => Colors {
                    accent: Color::Yellow,
                    on_accent: Color::Black,
                    neutral: Color::DarkGray,
                    syntax_highlighting: "base16-eighties.dark",
                },
                terminal_colorsaurus::ThemeMode::Light => Colors {
                    accent: Color::LightBlue,
                    on_accent: Color::Black,
                    neutral: Color::DarkGray,
                    syntax_highlighting: "InspiredGitHub",
                },
            };

        let mut title_line = Line::default().centered();
        title_line.push_span(Span::styled("lazycard", Style::new().fg(colors.neutral)));

        Self {
            route: Route::Review,
            pages: Pages::new(external_editor),
            db: database,
            colors,
            markup: Markup::new(),
            matcher: Matcher::new(),
            title_line,
            nav_line: Line::default().centered(),
            menu_line: Line::default().centered(),
            shortcuts: Shortcuts::new(),
        }
    }

    pub fn run(mut self, mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
        self.pages.review.on_enter(&self.db);
        self.render(&mut terminal)?;

        loop {
            let action = match crossterm::event::read()? {
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
                                    &mut self.db,
                                ),
                                Route::Editor(_) => self.pages.editor.on_input(
                                    key.code,
                                    key.modifiers,
                                    &mut self.markup,
                                    &mut self.db,
                                    &mut terminal,
                                )?,
                                Route::Cards => self.pages.cards.on_input(
                                    key.code,
                                    key.modifiers,
                                    &mut self.markup,
                                    &mut self.db,
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
                        Route::Review => self.pages.review.on_enter(&self.db),
                        Route::Editor(id) => {
                            self.pages.editor.on_enter(id, &self.db, &mut terminal)?
                        }
                        Route::Cards => self.pages.cards.on_enter(&mut self.db),
                    }

                    self.render(&mut terminal)?;
                }
                Action::Quit => {
                    break;
                }
            }
        }

        // Quitting app
        self.db.save()?;

        Ok(())
    }

    fn render<'a>(&'a mut self, terminal: &'a mut Terminal) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let [title_area, _, nav_area, menu_area, body_area, shortcuts_area] =
                Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(3),
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
                    Style::new().bold().fg(self.colors.accent)
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
                        &self.colors,
                        &mut self.menu_line,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(
                        body,
                        buf,
                        &self.colors,
                        &mut self.menu_line,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Cards => {
                    self.pages.cards.on_render(
                        body,
                        buf,
                        &self.db,
                        &self.colors,
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
            self.shortcuts.extend(
                ShortcutLine::Bottom,
                [
                    Shortcut::new("Next", "Tab"),
                    Shortcut::new("Prev", "⇧Tab"),
                    Shortcut::new("Quit", "Esc"),
                ],
            );
            self.shortcuts.render(shortcuts_area, buf, &self.colors);
        })
    }
}

pub struct Shortcut<'a> {
    name: &'a str,
    key: &'a str,
}

impl<'a> Shortcut<'a> {
    pub const fn new(name: &'a str, key: &'a str) -> Self {
        Self { name, key }
    }
}

pub struct Shortcuts<'a> {
    shortcuts: Vec<(ShortcutLine, Shortcut<'a>)>,
    top: Line<'a>,
    middle: Line<'a>,
    bottom: Line<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum ShortcutLine {
    Top,
    Middle,
    Bottom,
}

impl<'a> Shortcuts<'a> {
    fn new() -> Self {
        Self {
            shortcuts: Vec::new(),
            top: Line::default().centered(),
            middle: Line::default().centered(),
            bottom: Line::default().centered(),
        }
    }

    pub fn push(&mut self, line: ShortcutLine, shortcut: Shortcut<'a>) {
        self.shortcuts.push((line, shortcut));
    }

    pub fn extend(
        &mut self,
        line: ShortcutLine,
        shortcuts: impl IntoIterator<Item = Shortcut<'a>>,
    ) {
        self.shortcuts
            .extend(shortcuts.into_iter().map(|s| (line, s)));
    }

    fn render(&mut self, mut area: Rect, buf: &mut Buffer, colors: &Colors) {
        let key_color = colors.accent;
        for (line, shortcut) in self.shortcuts.drain(..) {
            let spans = [
                Span::raw(" "),
                Span::styled(shortcut.key, key_color),
                Span::raw(" "),
                Span::raw(shortcut.name),
                Span::raw(" "),
            ];
            match line {
                ShortcutLine::Top => self.top.extend(spans),
                ShortcutLine::Middle => self.middle.extend(spans),
                ShortcutLine::Bottom => self.bottom.extend(spans),
            }
        }

        (&self.top).render(area, buf);
        area.y += 1;
        (&self.middle).render(area, buf);
        area.y += 1;
        (&self.bottom).render(area, buf);

        self.top.spans.clear();
        self.middle.spans.clear();
        self.bottom.spans.clear();
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

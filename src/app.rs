use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::WidgetRef, CompletedFrame, DefaultTerminal};

use crate::{database::*, markup::Markup, pages::*, utils::*};

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
    shortcuts_line: Line<'static>,
    shortcuts_line2: Line<'static>,
    footer_line: Line<'static>,
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
    pub fn new(database: Database) -> Self {
        let colors =
            match terminal_colorsaurus::color_scheme(terminal_colorsaurus::QueryOptions::default())
                .unwrap_or_default()
            {
                terminal_colorsaurus::ColorScheme::Dark => Colors {
                    accent: Color::Yellow,
                    on_accent: Color::Black,
                    neutral: Color::DarkGray,
                    syntax_highlighting: "base16-eighties.dark",
                },
                terminal_colorsaurus::ColorScheme::Light => Colors {
                    accent: Color::LightBlue,
                    on_accent: Color::Black,
                    neutral: Color::DarkGray,
                    syntax_highlighting: "InspiredGitHub",
                },
            };

        let mut title_line = Line::default().centered();
        title_line.push_span(Span::styled("lazycard", STYLE_NONE.fg(colors.neutral)));

        let mut footer_line = Line::default().centered();
        footer_line.extend(Shortcut::new("Next", "Tab").as_spans(colors.accent));
        footer_line.extend(Shortcut::new("Prev", "⇧Tab").as_spans(colors.accent));
        footer_line.extend(Shortcut::new("Quit", "Esc").as_spans(colors.accent));

        Self {
            route: Route::Review,
            pages: Pages::new(),
            db: database,
            colors,
            markup: Markup::new(),
            matcher: Matcher::new(),
            title_line,
            nav_line: Line::default().centered(),
            menu_line: Line::default().centered(),
            shortcuts: Shortcuts::new(),
            shortcuts_line: Line::default().centered(),
            shortcuts_line2: Line::default().centered(),
            footer_line,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
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
                                ),
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
                        Route::Editor(id) => self.pages.editor.on_enter(id, &self.db),
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

    fn render<'a>(
        &'a mut self,
        terminal: &'a mut DefaultTerminal,
    ) -> std::io::Result<CompletedFrame> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let [title, _, nav, _, menu, body, shortcuts, shortcuts2, footer] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

            // Title
            self.title_line.render_ref(title, buf);

            // Navigation
            for route in [Route::Review, Route::Editor(None), Route::Cards] {
                let (name, is_current) = match route {
                    Route::Review => ("Review", matches!(self.route, Route::Review)),
                    Route::Editor(_) => ("Editor", matches!(self.route, Route::Editor(_))),
                    Route::Cards => ("Cards", matches!(self.route, Route::Cards)),
                };
                let style = if is_current {
                    STYLE_BOLD.fg(self.colors.accent)
                } else {
                    STYLE_NONE
                };
                self.nav_line
                    .extend([Span::styled(name, style), Span::raw("   ")]);
            }
            self.nav_line.spans.pop();
            self.nav_line.render_ref(nav, buf);
            self.nav_line.spans.clear();

            // Body
            const MAX_WIDTH: u16 = 64;
            let body = layout_center_horizontal(body, Constraint::Length(MAX_WIDTH))
                .inner(Margin::new(1, 1));
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
            self.menu_line.render_ref(menu, buf);
            self.menu_line.spans.clear();

            // Shortcuts
            self.shortcuts
                .drain_first()
                .for_each(|s| self.shortcuts_line.extend(s.as_spans(self.colors.accent)));
            self.shortcuts_line.render_ref(shortcuts, buf);
            self.shortcuts_line.spans.clear();

            self.shortcuts
                .drain_second()
                .for_each(|s| self.shortcuts_line2.extend(s.as_spans(self.colors.accent)));
            self.shortcuts_line2.render_ref(shortcuts2, buf);
            self.shortcuts_line2.spans.clear();

            // Footer
            self.footer_line.render_ref(footer, buf);
        })
    }
}

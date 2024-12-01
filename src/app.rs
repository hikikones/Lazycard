use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::WidgetRef, CompletedFrame, DefaultTerminal};

use crate::{database::*, markup::Markup, pages::*, utils::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
    colors: Colors,
    markup: Markup,
    menu: Menu<'static>,
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

        Self {
            running: true,
            route: Route::Review,
            pages: Pages::new(),
            db: database,
            colors,
            markup: Markup::new(),
            menu: Menu::new(),
            shortcuts: Shortcuts::new(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        self.pages.review.on_enter(&self.db);
        self.render(&mut terminal)?;

        while self.running {
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
                    self.running = false;
                }
            }
        }

        Ok(())
    }

    fn render<'a>(
        &'a mut self,
        terminal: &'a mut DefaultTerminal,
    ) -> std::io::Result<CompletedFrame> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let [title, _, nav, _, menu, body, shortcuts, footer] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

            // Title
            let mut title_line = Line::default().alignment(Alignment::Center);
            title_line.push_span(Span::styled("lazycard", STYLE_NONE.fg(self.colors.neutral)));
            title_line.render(title, buf);

            // Navigation
            let mut nav_line = Line::default().alignment(Alignment::Center);
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
                nav_line.push_span(Span::styled(name, style));
                nav_line.push_span(Span::raw("   "));
            }
            nav_line.spans.pop();
            nav_line.render(nav, buf);

            // Body
            let body = layout_center_horizontal(body, Constraint::Length(64));
            let body = body.inner(MARGIN_CONTENT);
            match self.route {
                Route::Review => {
                    self.pages.review.on_render(
                        body,
                        buf,
                        &mut self.menu,
                        &self.colors,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(
                        body,
                        buf,
                        &mut self.menu,
                        &self.colors,
                        &mut self.markup,
                        &mut self.shortcuts,
                    );
                }
                Route::Cards => {
                    self.pages.cards.on_render(
                        body,
                        buf,
                        &mut self.menu,
                        &self.colors,
                        &mut self.markup,
                        &self.db,
                        &mut self.shortcuts,
                    );
                }
            }

            // Menu
            self.menu.render_ref(menu, buf);
            self.menu.spans.clear();

            // Shortcuts
            let mut shortcuts_line = Line::default().alignment(Alignment::Center);
            for shortcut in self.shortcuts.drain(..) {
                shortcuts_line.extend(shortcut.as_spans(self.colors.accent));
            }
            shortcuts_line.render(shortcuts, buf);

            // Footer
            let mut footer_line = Line::default().alignment(Alignment::Center);
            footer_line.extend(SHORTCUT_NEXT.as_spans(self.colors.accent));
            footer_line.extend(SHORTCUT_PREV.as_spans(self.colors.accent));
            footer_line.extend(SHORTCUT_QUIT.as_spans(self.colors.accent));
            footer_line.render(footer, buf);
        })
    }
}

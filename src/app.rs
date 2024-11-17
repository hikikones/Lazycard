use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, CompletedFrame, DefaultTerminal};

use crate::{database::*, markup::Markup, pages::*, utils::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
    markup: Markup,
    shortcuts: Shortcuts<'static>,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Quit,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            route: Route::Review,
            pages: Pages::new(),
            db: Database::new(),
            markup: Markup::new(),
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
                                Route::Cards => self.pages.cards.on_input(key.code, key.modifiers),
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
                        Route::Cards => self.pages.cards.on_enter(&self.db),
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

            let [title, _, nav, body, shortcuts, footer] = Layout::vertical([
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
            title_line.push_span(Span::styled("lazycard", STYLE_LABEL));
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
                    Style::new().bold()
                } else {
                    Style::new()
                };
                nav_line.push_span(Span::styled(name, style));
                nav_line.push_span(Span::raw("   "));
            }
            nav_line.spans.pop();
            nav_line.render(nav, buf);

            // Body
            let body = layout_center_horizontal(body, Constraint::Length(64));
            match self.route {
                Route::Review => {
                    self.pages.review.on_render(body, buf, &mut self.markup);
                    self.pages.review.shortcuts(&mut self.shortcuts);
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(body, buf, &mut self.markup);
                    self.pages.editor.shortcuts(&mut self.shortcuts);
                }
                Route::Cards => {
                    self.pages
                        .cards
                        .on_render(body, buf, &mut self.markup, &self.db);
                    self.pages.cards.shortcuts(&mut self.shortcuts);
                }
            }

            // Shortcuts
            let mut shortcuts_line = Line::default().alignment(Alignment::Center);
            for shortcut in self.shortcuts.drain(..) {
                shortcuts_line.extend(shortcut.as_spans());
            }
            shortcuts_line.render(shortcuts, buf);

            // Footer
            let mut footer_line = Line::default().alignment(Alignment::Center);
            footer_line.extend(SHORTCUT_NEXT.as_spans());
            footer_line.extend(SHORTCUT_PREV.as_spans());
            footer_line.extend(SHORTCUT_QUIT.as_spans());
            footer_line.render(footer, buf);
        })
    }
}

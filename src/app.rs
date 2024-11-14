use crossterm::event::{Event, KeyEventKind};
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
                        match self.route {
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
                    }

                    self.route = route;
                    self.markup.clear();

                    match route {
                        Route::Review => self.pages.review.on_enter(&self.db),
                        Route::Editor(id) => self.pages.editor.on_enter(id, &self.db),
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

            let [header, body, footer] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .areas(area);

            // Header
            Line::raw(format!("Lazycard: {}", self.route.title()))
                .alignment(Alignment::Center)
                .render(header, buf);

            // Body
            let body =
                layout_center_horizontal(body.inner(Margin::new(2, 2)), Constraint::Length(64));
            match self.route {
                Route::Review => {
                    self.pages.review.on_render(body, buf, &mut self.markup);
                    self.pages.review.shortcuts(&mut self.shortcuts);
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(body, buf, &mut self.markup);
                    self.pages.editor.shortcuts(&mut self.shortcuts);
                }
            }

            // Footer
            let mut footer_line = Line::default();
            for shortcut in self.shortcuts.drain(..) {
                footer_line.extend([
                    Span::styled(shortcut.key, STYLE_LABEL),
                    Span::raw(" "),
                    Span::raw(shortcut.name),
                    Span::raw("  "),
                ]);
            }
            footer_line.spans.pop();
            footer_line.alignment(Alignment::Center).render(footer, buf);
        })
    }
}

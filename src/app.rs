use super::*;

pub struct App {
    running: bool,
    route: Route,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            route: Route::Review,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            let current_route = self.route;
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => self.running = false,
                        KeyCode::Tab => match current_route {
                            Route::Review => self.route = Route::AddCard,
                            Route::AddCard => self.route = Route::Review,
                            Route::EditCard => {}
                        },
                        _ => {
                            if let Some(route) = match current_route {
                                Route::Review => Review::new().input(key.code),
                                Route::AddCard => AddCard::new().input(key.code),
                                Route::EditCard => EditCard::new().input(key.code),
                            } {
                                self.route = route;
                            }
                        }
                    }
                }
            }

            if self.route != current_route {
                //todo
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);
        let body = center_horizontal(body.inner(Margin::new(2, 2)), Constraint::Length(64));

        match self.route {
            Route::Review => {
                Line::from("Lazycard — Review")
                    .centered()
                    .render(header, buf);
                Review::new().render(body, buf);
                Line::from("ESC to quit  TAB to menu  'e' to edit")
                    .centered()
                    .render(footer, buf);
            }
            Route::AddCard => {
                Line::from("Lazycard — Add Card")
                    .centered()
                    .render(header, buf);
                AddCard::new().render(body, buf);
                Line::from("ESC to quit  TAB to menu  '^s' to save")
                    .centered()
                    .render(footer, buf);
            }
            Route::EditCard => {
                Line::from("Lazycard — Edit Card")
                    .centered()
                    .render(header, buf);
                EditCard::new().render(body, buf);
                Line::from("ESC to quit  '^s' to save '^c' to cancel")
                    .centered()
                    .render(footer, buf);
            }
        }
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    center_vertical(center_horizontal(area, horizontal), vertical)
}

fn center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn center_vertical(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

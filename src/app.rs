use crossterm::event::{Event, KeyCode, KeyEventKind};
use layout::Flex;
use ratatui::{prelude::*, DefaultTerminal};

use crate::{database::*, navigation::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
}

impl App {
    pub fn new() -> Self {
        let mut db = Database::new();
        db.insert(CardId(1), Card::new("first card"));
        db.insert(CardId(2), Card::new("second card"));

        Self {
            running: true,
            route: Route::Review,
            pages: Pages::new(),
            db,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        self.pages.review.enter(&self.db);

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
                                Route::Review => self.pages.review.input(key, &mut self.db),
                                Route::AddCard => self.pages.add_card.input(key, &mut self.db),
                                Route::EditCard => self.pages.edit_card.input(key, &mut self.db),
                            } {
                                self.route = route;
                            }
                        }
                    }
                }
            }

            if current_route != self.route {
                match current_route {
                    Route::Review => self.pages.review.exit(),
                    Route::AddCard => self.pages.add_card.exit(),
                    Route::EditCard => self.pages.edit_card.exit(),
                }

                match self.route {
                    Route::Review => self.pages.review.enter(&self.db),
                    Route::AddCard => self.pages.add_card.enter(&self.db),
                    Route::EditCard => self.pages.edit_card.enter(&self.db),
                }
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
                self.pages.review.render(body, buf);
                Line::from("ESC to quit  TAB to menu  'e' to edit")
                    .centered()
                    .render(footer, buf);
            }
            Route::AddCard => {
                Line::from("Lazycard — Add Card")
                    .centered()
                    .render(header, buf);
                self.pages.add_card.render(body, buf);
                Line::from("ESC to quit  TAB to menu  '^s' to save")
                    .centered()
                    .render(footer, buf);
            }
            Route::EditCard => {
                Line::from("Lazycard — Edit Card")
                    .centered()
                    .render(header, buf);
                self.pages.edit_card.render(body, buf);
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

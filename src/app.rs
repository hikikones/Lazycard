use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use layout::Flex;
use ratatui::{prelude::*, DefaultTerminal};

use crate::{database::*, navigation::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
}

pub enum Message {
    Render,
    Route(Route),
    Quit,
}

pub enum InputEvent {
    Key(KeyEvent),
    Paste(String),
}

impl App {
    pub fn new() -> Self {
        let mut db = Database::new();
        db.insert(CardId(1), Card::new("first card"));
        db.insert(CardId(2), Card::new("second card"));
        db.insert(CardId(3), Card::new("third card"));

        Self {
            running: true,
            route: Route::Review,
            pages: Pages::new(),
            db,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        self.pages.review.enter(&self.db);
        terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

        while self.running {
            let message = match crossterm::event::read()? {
                Event::Key(key) => {
                    let ev = InputEvent::Key(key);
                    let db = &mut self.db;
                    match self.route {
                        Route::Review => self.pages.review.input(ev, db),
                        Route::AddCard => self.pages.add_card.input(ev, db),
                        Route::EditCard => self.pages.edit_card.input(ev, db),
                    }
                }
                Event::Paste(s) => {
                    let ev = InputEvent::Paste(s);
                    let db = &mut self.db;
                    match self.route {
                        Route::Review => self.pages.review.input(ev, db),
                        Route::AddCard => self.pages.add_card.input(ev, db),
                        Route::EditCard => self.pages.edit_card.input(ev, db),
                    }
                }
                Event::Resize(_, _) => Some(Message::Render), // todo: redraw
                _ => None,
            };

            if let Some(message) = message {
                match message {
                    Message::Render => {
                        terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
                    }
                    Message::Route(route) => {
                        match self.route {
                            Route::Review => self.pages.review.exit(),
                            Route::AddCard => self.pages.add_card.exit(),
                            Route::EditCard => self.pages.edit_card.exit(),
                        }

                        self.route = route;

                        match route {
                            Route::Review => self.pages.review.enter(&self.db),
                            Route::AddCard => self.pages.add_card.enter(&self.db),
                            Route::EditCard => self.pages.edit_card.enter(&self.db),
                        }

                        terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
                    }
                    Message::Quit => {
                        self.running = false;
                    }
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

fn _center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    _center_vertical(center_horizontal(area, horizontal), vertical)
}

fn center_horizontal(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

fn _center_vertical(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

use crossterm::event::{Event, KeyEvent};
use layout::Flex;
use ratatui::{prelude::*, CompletedFrame, DefaultTerminal};

use crate::{database::*, navigation::*};

pub struct App {
    running: bool,
    route: Route,
    pages: Pages,
    db: Database,
}

pub enum Message {
    None,
    Render,
    Route(Route),
    Quit,
}

pub enum InputEvent {
    Key(KeyEvent),
    Paste(String),
}

pub struct Areas {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
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
        self.render(&mut terminal)?;

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
                Event::Resize(_, _) => Message::Render,
                _ => Message::None,
            };

            match message {
                Message::None => {}
                Message::Render => {
                    self.render(&mut terminal)?;
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

                    self.render(&mut terminal)?;
                }
                Message::Quit => {
                    self.running = false;
                }
            }
        }

        Ok(())
    }

    fn render<'a>(&'a self, terminal: &'a mut DefaultTerminal) -> std::io::Result<CompletedFrame> {
        terminal.draw(|frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();

            let [header, body, footer] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .areas(area);
            let body = center_horizontal(body.inner(Margin::new(2, 2)), Constraint::Length(64));

            let areas = Areas {
                header,
                body,
                footer,
            };

            match self.route {
                Route::Review => self.pages.review.render(&areas, buf),
                Route::AddCard => self.pages.add_card.render(&areas, buf),
                Route::EditCard => self.pages.edit_card.render(&areas, buf),
            }
        })
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

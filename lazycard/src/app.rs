use std::path::PathBuf;

use database::Database;
use ratatui::{
    CompletedFrame,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Style},
};
use widgets::{CellSize, KittyGraphics, Markup, Shortcut, Shortcuts, TextSegment};

use crate::{pages::*, settings::Settings, symbols, terminal::Terminal};

pub struct App {
    route: Route,
    state: AppState,
    pages: Pages,
    database: Database,
    settings: Settings,
    markup: Markup,
    kitty: KittyGraphics,
    // matcher: Matcher,
    text: TextSegment,
    shortcuts: Shortcuts,
}

enum AppState {
    Route,
    Search,
    Logs,
}

pub enum Action {
    None,
    Render,
    Route(Route),
    Log(Log),
    Quit,
}

pub struct AppInput(KeyEvent);

impl AppInput {
    pub const fn key_pressed(&self) -> KeyCode {
        self.0.code
    }

    pub const fn _key_modifiers(&self) -> KeyModifiers {
        self.0.modifiers
    }

    pub const fn key_pressed_and_modifiers(&self) -> (KeyCode, KeyModifiers) {
        (self.0.code, self.0.modifiers)
    }
}

impl App {
    pub fn new(database: Database, cell_size: CellSize, settings_path: Option<PathBuf>) -> Self {
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

        let colors = settings.colors();
        let pages = Pages {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(colors),
            cards: CardsPage::new(colors),
            search: SearchPage::new(colors),
            logs,
        };

        Self {
            route: Route::Review,
            state: AppState::Route,
            pages,
            database,
            markup: Markup::new(settings.syntax_highlighting()),
            kitty: KittyGraphics::new(cell_size),
            // matcher: Matcher::new(),
            text: TextSegment::new().with_alignment(Alignment::Center),
            shortcuts: Shortcuts::new().with_colors(Color::Reset, settings.primary()),
            settings,
        }
    }

    pub fn run(&mut self, mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
        self.pages.review.on_enter(&self.database, &mut self.markup);
        self.render(&mut terminal)?;

        loop {
            let action = match ratatui::crossterm::event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Esc => Action::Quit,
                            KeyCode::Tab | KeyCode::BackTab => match self.state {
                                AppState::Route => {
                                    let next_route = if key.code == KeyCode::Tab {
                                        self.route.next()
                                    } else {
                                        self.route.prev()
                                    };
                                    Action::Route(next_route)
                                }
                                AppState::Search => {
                                    self.state = AppState::Route;
                                    self.pages.search.on_exit();
                                    Action::Render
                                }
                                AppState::Logs => {
                                    self.state = AppState::Route;
                                    self.pages.logs.on_exit();
                                    Action::Render
                                }
                            },
                            KeyCode::Char('f') => {
                                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                                if ctrl {
                                    match self.state {
                                        AppState::Route => {
                                            self.state = AppState::Search;
                                            self.pages
                                                .search
                                                .on_enter(&self.database, &mut self.markup);
                                        }
                                        AppState::Search => {
                                            self.state = AppState::Route;
                                            self.pages.search.on_exit();
                                        }
                                        AppState::Logs => {
                                            self.state = AppState::Search;
                                            self.pages.logs.on_exit();
                                            self.pages
                                                .search
                                                .on_enter(&self.database, &mut self.markup);
                                        }
                                    }
                                    Action::Render
                                } else {
                                    self.on_input(AppInput(key), &mut terminal)
                                }
                            }
                            KeyCode::Char('l') => {
                                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                                if ctrl && !self.pages.logs.is_empty() {
                                    match self.state {
                                        AppState::Route => {
                                            self.state = AppState::Logs;
                                            self.pages.logs.on_enter();
                                        }
                                        AppState::Search => {
                                            self.state = AppState::Logs;
                                            self.pages.search.on_exit();
                                            self.pages.logs.on_enter();
                                        }
                                        AppState::Logs => {
                                            self.state = AppState::Route;
                                            self.pages.logs.on_exit();
                                        }
                                    }
                                    Action::Render
                                } else {
                                    self.on_input(AppInput(key), &mut terminal)
                                }
                            }
                            _ => self.on_input(AppInput(key), &mut terminal),
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
                        Route::Review => {
                            self.pages.review.on_enter(&self.database, &mut self.markup)
                        }
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

    pub fn quit(self) -> Result<(), Box<dyn std::error::Error>> {
        //todo?
        Ok(())
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

            // Clear any rendered image from markup and reset max items
            self.markup.delete_images(&self.kitty).unwrap();
            self.markup.set_max_items(None);

            // Body
            const MAX_WIDTH: u16 = 64;
            const MARGIN: u16 = 1;
            let body = body_area
                .centered_horizontally(Constraint::Length(MAX_WIDTH + MARGIN))
                .inner(Margin::new(MARGIN, MARGIN));
            match self.state {
                AppState::Route => match self.route {
                    Route::Review => {
                        self.pages.review.on_render(
                            body,
                            buf,
                            colors,
                            &self.database,
                            &mut self.text,
                            &mut self.markup,
                            &mut self.kitty,
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
                            &mut self.kitty,
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
                            &mut self.kitty,
                            &mut self.shortcuts,
                        );
                    }
                },
                AppState::Search => {
                    self.pages.search.on_render(
                        body,
                        buf,
                        &self.database,
                        &mut self.text,
                        &mut self.markup,
                        &mut self.kitty,
                        &mut self.shortcuts,
                        colors,
                    );
                }
                AppState::Logs => {
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
                Shortcut::new("Find", symbols::ctrl!("f")),
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

    fn on_input(&mut self, input: AppInput, terminal: &mut Terminal) -> Action {
        match self.state {
            AppState::Route => match self.route {
                Route::Review => {
                    self.pages
                        .review
                        .on_input(input, &mut self.markup, &mut self.database)
                }
                Route::Editor(_) => self.pages.editor.on_input(
                    input,
                    &mut self.markup,
                    &mut self.database,
                    terminal,
                ),
                Route::Cards => {
                    self.pages
                        .cards
                        .on_input(input, &mut self.markup, &mut self.database)
                }
            },
            AppState::Search => {
                match self
                    .pages
                    .search
                    .on_input(input, &self.database, &mut self.markup)
                {
                    SearchAction::None => Action::None,
                    SearchAction::Render => Action::Render,
                    SearchAction::Edit(id) => {
                        self.state = AppState::Route;
                        self.pages.search.on_exit();
                        Action::Route(Route::Editor(id))
                    }
                    SearchAction::Goto(id) => {
                        self.state = AppState::Route;
                        self.pages.search.on_exit();
                        todo!("goto cards")
                    }
                }
            }
            AppState::Logs => match self.pages.logs.on_input(input) {
                LogsAction::None => Action::None,
                LogsAction::Render => Action::Render,
                LogsAction::Done => {
                    self.state = AppState::Route;
                    self.pages.logs.on_exit();
                    Action::Render
                }
            },
        }
    }
}

// pub struct Matcher {
//     matcher: nucleo_matcher::Matcher,
//     pattern: nucleo_matcher::pattern::Pattern,
//     buffer: Vec<char>,
// }

// impl Matcher {
//     pub fn new() -> Self {
//         Self {
//             matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
//             pattern: nucleo_matcher::pattern::Pattern::new(
//                 "",
//                 nucleo_matcher::pattern::CaseMatching::Smart,
//                 nucleo_matcher::pattern::Normalization::Smart,
//                 nucleo_matcher::pattern::AtomKind::Fuzzy,
//             ),
//             buffer: Vec::new(),
//         }
//     }

//     pub fn update(&mut self, pattern: &str) {
//         self.pattern.reparse(
//             pattern,
//             nucleo_matcher::pattern::CaseMatching::Smart,
//             nucleo_matcher::pattern::Normalization::Smart,
//         );
//     }

//     pub fn score(&mut self, haystack: &str) -> Option<u32> {
//         self.pattern.score(
//             nucleo_matcher::Utf32Str::new(haystack, &mut self.buffer),
//             &mut self.matcher,
//         )
//     }
// }

// todo: remove
// fn center_horizontal(area: Rect, constraint: Constraint) -> Rect {
//     let [area] = Layout::horizontal([constraint])
//         .flex(Flex::Center)
//         .areas(area);
//     area
// }

// pub trait CardsIterExt<'a> {
//     fn due(self) -> impl Iterator<Item = (CardId, &'a Card)>;
//     fn active(self) -> impl Iterator<Item = (CardId, &'a Card)>;
//     fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)>;
//     fn search(
//         self,
//         pattern: &str,
//         matcher: &'a mut Matcher,
//     ) -> impl Iterator<Item = (CardId, &'a Card, u32)>;
// }

// impl<'a, I> CardsIterExt<'a> for I
// where
//     I: Iterator<Item = (CardId, &'a Card)>,
// {
//     fn due(self) -> impl Iterator<Item = (CardId, &'a Card)> {
//         let now = UnixTime::now();
//         self.filter(move |(_, card)| !card.archived && card.is_due(now))
//     }

//     fn active(self) -> impl Iterator<Item = (CardId, &'a Card)> {
//         self.filter(|(_, card)| !card.archived)
//     }

//     fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)> {
//         self.filter(|(_, card)| card.archived)
//     }

//     fn search(
//         self,
//         pattern: &str,
//         matcher: &'a mut Matcher,
//     ) -> impl Iterator<Item = (CardId, &'a Card, u32)> {
//         matcher.update(pattern);
//         self.filter_map(|(id, card)| {
//             matcher
//                 .score(card.content.as_str())
//                 .map(|score| (id, card, score))
//         })
//     }
// }

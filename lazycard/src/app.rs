use std::path::PathBuf;

use database::Database;
use ratatui::{
    CompletedFrame,
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
};
use widgets::{CellSize, KittyGraphics, Markup, Shortcut, Shortcuts, TextSegment};

use crate::{
    pages::*,
    settings::{Colors, Settings},
    symbols,
    terminal::Terminal,
};

pub struct App {
    route: Route,
    state: AppState,
    pages: Pages,
    database: Database,
    settings: Settings,
    markup: Markup,
    kitty: KittyGraphics,
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
    ApplySettings,
    Quit,
}

pub struct AppInput(KeyEvent);

impl AppInput {
    pub const fn key_pressed(&self) -> KeyCode {
        self.0.code
    }

    pub const fn key_pressed_and_modifiers(self) -> (KeyCode, KeyModifiers) {
        (self.0.code, self.0.modifiers)
    }
}

pub struct AppRender<'a> {
    area: Rect,
    buffer: &'a mut Buffer,
}

impl<'a> AppRender<'a> {
    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn buffer(self) -> &'a mut Buffer {
        self.buffer
    }

    pub const fn area_and_buffer(self) -> (Rect, &'a mut Buffer) {
        (self.area, self.buffer)
    }
}

impl App {
    pub fn new(
        database: Database,
        cell_size: CellSize,
        assets_dir: PathBuf,
        settings_path: Option<PathBuf>,
    ) -> Self {
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
            cards: CardsPage::new(),
            tags: TagsPage::new(&database, colors),
            settings: SettingsPage::new(&settings),
            search: SearchPage::new(colors),
            logs,
        };

        Self {
            route: Route::default(),
            state: AppState::Route,
            pages,
            database,
            markup: Markup::new(assets_dir, settings.syntax_highlighting()),
            kitty: KittyGraphics::new(cell_size),
            text: TextSegment::new().with_alignment(Alignment::Center),
            shortcuts: Shortcuts::new(),
            settings,
        }
    }

    pub fn run(&mut self, mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
        // Apply settings
        self.apply_settings();

        // Render default page
        self.on_enter();
        self.render(&mut terminal)?;

        // Run event loop
        loop {
            let action = match ratatui::crossterm::event::read()? {
                Event::Key(key_ev) => {
                    if key_ev.kind == KeyEventKind::Press {
                        match key_ev.code {
                            KeyCode::Esc => Action::Quit,
                            KeyCode::Tab | KeyCode::BackTab => match self.state {
                                AppState::Route => {
                                    let next_route = if key_ev.code == KeyCode::Tab {
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
                                let ctrl = key_ev.modifiers.contains(KeyModifiers::CONTROL);
                                if ctrl {
                                    match self.state {
                                        AppState::Route => {
                                            self.state = AppState::Search;
                                            self.pages.search.on_enter(&self.database);
                                        }
                                        AppState::Search => {
                                            self.state = AppState::Route;
                                            self.pages.search.on_exit();
                                        }
                                        AppState::Logs => {
                                            self.state = AppState::Search;
                                            self.pages.logs.on_exit();
                                            self.pages.search.on_enter(&self.database);
                                        }
                                    }
                                    Action::Render
                                } else {
                                    self.on_input(key_ev, &mut terminal)
                                }
                            }
                            KeyCode::Char('l') => {
                                let ctrl = key_ev.modifiers.contains(KeyModifiers::CONTROL);
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
                                    self.on_input(key_ev, &mut terminal)
                                }
                            }
                            _ => self.on_input(key_ev, &mut terminal),
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
                    self.on_exit();
                    self.route = route;
                    self.markup.clear();
                    self.on_enter();
                    self.render(&mut terminal)?;
                }
                Action::Log(log) => {
                    self.pages.logs.enqueue(log);
                    self.render(&mut terminal)?;
                }
                Action::ApplySettings => {
                    self.apply_settings();
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

    const fn apply_settings(&mut self) {
        self.pages
            .review
            .set_desired_retention(self.settings.desired_retention_as_fraction());
    }

    fn render<'a>(&'a mut self, terminal: &'a mut Terminal) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let mut area = frame.area();
            let buf = frame.buffer_mut();

            let colors = &self.settings.colors().clone();

            // Navigation
            if area.height > 0 {
                const SPACING: &str = "   ";
                for (route, name, spacing) in [
                    (Route::Review, "Review", SPACING),
                    (Route::Editor(None), "Editor", SPACING),
                    (Route::Cards(None), "Cards", SPACING),
                    (Route::Tags, "Tags", SPACING),
                    (Route::Settings, "Settings", ""),
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
                self.text.render(area, buf);
                self.text.clear();

                area.height = area.height.saturating_sub(1);
                area.y += 1;
            }

            // Clear any rendered image from markup and reset max items
            self.markup.delete_images(&self.kitty).unwrap();
            self.markup.set_max_items(None);

            // Page content
            if area.height > 0 {
                self.shortcuts.set_colors(Color::Reset, colors.secondary);

                const MAX_WIDTH: u16 = 64;
                const MARGIN: u16 = 1;
                const SHORTCUTS_HEIGHT: u16 = 2;

                let height_removal = if area.height > SHORTCUTS_HEIGHT * 4 {
                    SHORTCUTS_HEIGHT
                } else {
                    0
                };

                let body = widgets::align(
                    Rect {
                        width: area.width.min(MAX_WIDTH + MARGIN * 2),
                        height: area.height.saturating_sub(height_removal),
                        ..area
                    }
                    .inner(Margin::new(MARGIN, MARGIN)),
                    area,
                    widgets::Alignment::CenterHorizontal,
                );
                self.on_render(body, buf, colors);

                let body_height = body.height + MARGIN * 2;
                area.height = area.height.saturating_sub(body_height);
                area.y += body_height;
            }

            // Page shortcuts
            if area.y > 0 {
                self.shortcuts.render(area, buf);
                self.shortcuts.clear();

                area.height = area.height.saturating_sub(1);
                area.y += 1;
            }

            // App shortcuts
            if area.y > 0 {
                self.shortcuts.set_colors(Color::Reset, colors.primary);
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

                self.shortcuts.render(area, buf);
                self.shortcuts.clear();
            }
        })
    }

    fn on_render(&mut self, body: Rect, buf: &mut Buffer, colors: &Colors) {
        let render = AppRender {
            area: body,
            buffer: buf,
        };

        match self.state {
            AppState::Route => match self.route {
                Route::Review => {
                    self.pages.review.on_render(
                        render,
                        &self.database,
                        colors,
                        &mut self.markup,
                        &mut self.kitty,
                        &mut self.shortcuts,
                    );
                }
                Route::Editor(_) => {
                    self.pages.editor.on_render(
                        render,
                        &self.database,
                        colors,
                        &mut self.markup,
                        &mut self.kitty,
                        &mut self.shortcuts,
                    );
                }
                Route::Cards(_) => {
                    self.pages.cards.on_render(
                        render,
                        &self.database,
                        colors,
                        &mut self.markup,
                        &mut self.kitty,
                        &mut self.shortcuts,
                    );
                }
                Route::Tags => {
                    self.pages
                        .tags
                        .on_render(render, colors, &mut self.shortcuts);
                }
                Route::Settings => {
                    self.pages
                        .settings
                        .on_render(render, &mut self.settings, &mut self.shortcuts);
                }
            },
            AppState::Search => {
                self.pages.search.on_render(
                    render,
                    colors,
                    &mut self.markup,
                    &mut self.kitty,
                    &mut self.shortcuts,
                );
            }
            AppState::Logs => {
                self.pages
                    .logs
                    .on_render(render, colors, &mut self.shortcuts);
            }
        }
    }

    fn on_enter(&mut self) {
        match self.route {
            Route::Review => self.pages.review.on_enter(&self.database, &mut self.markup),
            Route::Editor(id) => self.pages.editor.on_enter(id, &self.database),
            Route::Cards(id) => self.pages.cards.on_enter(&mut self.database, id),
            Route::Tags => self.pages.tags.on_enter(),
            Route::Settings => self.pages.settings.on_enter(),
        }
    }

    fn on_exit(&mut self) {
        match self.route {
            Route::Review => self.pages.review.on_exit(),
            Route::Editor(_) => self.pages.editor.on_exit(),
            Route::Cards(_) => self.pages.cards.on_exit(),
            Route::Tags => self.pages.tags.on_exit(),
            Route::Settings => self.pages.settings.on_exit(),
        }
    }

    fn on_input(&mut self, key_event: KeyEvent, terminal: &mut Terminal) -> Action {
        let input = AppInput(key_event);
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
                Route::Cards(_) => {
                    self.pages
                        .cards
                        .on_input(input, &mut self.markup, &mut self.database)
                }
                Route::Tags => self.pages.tags.on_input(input, &self.database),
                Route::Settings => self.pages.settings.on_input(input, &mut self.settings),
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
                        Action::Route(Route::Editor(Some(id)))
                    }
                    SearchAction::Goto(id) => {
                        self.state = AppState::Route;
                        self.pages.search.on_exit();
                        Action::Route(Route::Cards(Some(CardsRoute::Card(id))))
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

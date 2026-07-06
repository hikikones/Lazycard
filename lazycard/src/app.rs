use std::path::PathBuf;

use ratatui::{
    CompletedFrame,
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Margin, Rect},
    style::Color,
};
use widgets::{CellSize, KittyGraphics, Markup, Shortcut, Shortcuts};

use crate::{
    database::Database,
    pages::{Log, PageState, Pages, Route},
    settings::Settings,
    symbols,
    terminal::Terminal,
};

pub struct App {
    pages: Pages,
    database: Database,
    settings: Settings,
    markup: Markup,
    kitty: KittyGraphics,
    shortcuts: Shortcuts,
    is_running: bool,
}

pub enum Action {
    None,
    Render,
    Forward,
    Backward,
    Route(Route),
    ToggleSearch,
    ToggleLogs,
    EnqueueLog(Log),
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
        mut database: Database,
        cell_size: CellSize,
        assets_dir: PathBuf,
        settings_path: Option<PathBuf>,
    ) -> Self {
        let mut settings_err = None;

        let settings = Settings::read(settings_path.clone())
            .inspect_err(|err| {
                settings_err = Some(Log::new(err));
            })
            .unwrap_or_default()
            .with_path(settings_path);

        let mut markup = Markup::new(assets_dir, settings.syntax_highlight_theme());
        let mut pages = Pages::new(Route::DEFAULT, &settings, &mut database, &mut markup);

        if let Some(log) = settings_err {
            pages.enqueue_log(log);
        }

        Self {
            pages,
            database,
            markup,
            kitty: KittyGraphics::new(cell_size),
            shortcuts: Shortcuts::new(),
            settings,
            is_running: true,
        }
    }

    pub fn run(&mut self, mut terminal: Terminal) -> Result<(), Box<dyn std::error::Error>> {
        // Apply settings
        self.apply_settings();

        // Render default page
        self.render(&mut terminal)?;

        // Run event loop
        while self.is_running {
            let event = ratatui::crossterm::event::read()?;
            let action = self.read_event(event, &mut terminal);
            self.apply_action(action, &mut terminal)?;
        }

        Ok(())
    }

    pub fn quit(self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    const fn apply_settings(&mut self) {
        self.markup.set_scrollbar(self.settings.colors.scrollbar());
        self.pages.apply_settings(&self.settings);
    }

    fn read_event(&mut self, event: Event, terminal: &mut Terminal) -> Action {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return Action::None;
                }

                match key.code {
                    KeyCode::Esc => Action::Quit,
                    KeyCode::Tab => Action::Forward,
                    KeyCode::BackTab => Action::Backward,
                    KeyCode::Char('f') => {
                        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                        if ctrl {
                            Action::ToggleSearch
                        } else {
                            self.on_input(key, terminal)
                        }
                    }
                    KeyCode::Char('l') => {
                        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                        if ctrl && !self.pages.is_logs_empty() {
                            Action::ToggleLogs
                        } else {
                            self.on_input(key, terminal)
                        }
                    }
                    _ => self.on_input(key, terminal),
                }
            }
            Event::Resize(_, _) => {
                // TODO: Somehow pass a was_window_resized bool to markup render.
                Action::Render
            }
            _ => Action::None,
        }
    }

    fn apply_action(
        &mut self,
        action: Action,
        terminal: &mut Terminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            Action::None => {}
            Action::Render => {
                self.render(terminal)?;
            }
            Action::Forward => {
                self.set_page(self.pages.forward());
                self.render(terminal)?;
            }
            Action::Backward => {
                self.set_page(self.pages.backward());
                self.render(terminal)?;
            }
            Action::Route(route) => {
                self.set_page(PageState::Route(route));
                self.render(terminal)?;
            }
            Action::ToggleSearch => {
                self.set_page(PageState::Search);
                self.render(terminal)?;
            }
            Action::ToggleLogs => {
                self.set_page(PageState::Logs);
                self.render(terminal)?;
            }
            Action::EnqueueLog(log) => {
                self.pages.enqueue_log(log);
                self.render(terminal)?;
            }
            Action::ApplySettings => {
                self.apply_settings();
                self.render(terminal)?;
            }
            Action::Quit => {
                self.is_running = false;
            }
        }

        Ok(())
    }

    fn set_page(&mut self, state: PageState) {
        self.markup.clear();
        self.pages
            .set_state(state, &mut self.database, &mut self.markup);
    }

    fn render<'a>(&'a mut self, terminal: &'a mut Terminal) -> std::io::Result<CompletedFrame<'a>> {
        terminal.draw(|frame| {
            let mut area = frame.area();
            let buf = frame.buffer_mut();

            let colors = &self.settings.colors().clone();

            // Navigation
            if area.height > 0 {
                self.pages.render_navigation(area, buf, colors);

                area.height = area.height.saturating_sub(1);
                area.y += 1;
            }

            // Clear any rendered image from markup and reset max items
            self.markup.delete_images(&self.kitty)?;
            self.markup.set_max_items(None);

            // Page content
            if area.height > 0 {
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
                self.on_render(body, buf);

                let body_height = body.height + MARGIN * 2;
                area.height = area.height.saturating_sub(body_height);
                area.y += body_height;
            }

            // Page shortcuts
            if area.y > 0 {
                self.shortcuts
                    .set_colors(Color::Reset, colors.secondary)
                    .render(area, buf);
                self.shortcuts.clear();

                area.height = area.height.saturating_sub(1);
                area.y += 1;
            }

            // App shortcuts
            if area.y > 0 {
                self.shortcuts.extend([
                    Shortcut::new("Quit", symbols::ESCAPE),
                    Shortcut::new("Navigate", symbols::shift!(symbols::TAB)),
                    Shortcut::new("Find", symbols::ctrl!("f")),
                ]);

                if !self.pages.is_logs_empty() {
                    let key = symbols::ctrl!("l");
                    let new_logs = self.pages.logs_queue_len();
                    if new_logs > 0 {
                        utils::format_int(new_logs, |new_logs| {
                            self.shortcuts.push_iter(["Logs(", new_logs, ")"], key);
                        });
                    } else {
                        self.shortcuts.push(Shortcut::new("Logs", key));
                    }
                }

                self.shortcuts
                    .set_colors(Color::Reset, colors.primary)
                    .render(area, buf);
                self.shortcuts.clear();
            }

            Ok(())
        })
    }

    fn on_render(&mut self, body: Rect, buf: &mut Buffer) {
        let render = AppRender {
            area: body,
            buffer: buf,
        };
        self.pages.on_render(
            render,
            &self.settings,
            &mut self.database,
            &mut self.markup,
            &mut self.kitty,
            &mut self.shortcuts,
        );
    }

    fn on_input(&mut self, key_event: KeyEvent, terminal: &mut Terminal) -> Action {
        let input = AppInput(key_event);
        self.pages.on_input(
            input,
            &mut self.database,
            &mut self.markup,
            terminal,
            &mut self.settings,
        )
    }
}

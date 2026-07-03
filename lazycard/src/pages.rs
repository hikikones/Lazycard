mod cards;
mod editor;
mod logs;
mod review;
mod search;
mod settings;
mod tags;

pub use cards::*;
pub use editor::*;
pub use logs::*;
pub use review::*;
pub use search::*;
pub use settings::*;
pub use tags::*;

use database::{CardId, Database, TagId};
use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use widgets::{KittyGraphics, Markup, Shortcuts};

use crate::{
    app::{Action, AppInput, AppRender},
    settings::{Colors, Settings},
    terminal::Terminal,
};

pub struct Pages {
    review: ReviewPage,
    editor: CardEditorPage,
    cards: CardsPage,
    tags: TagsPage,
    settings: SettingsPage,
    search: SearchPage,
    logs: LogsPage,
    route: Route,
    state: PageState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<CardId>),
    Cards(Option<CardsParam>),
    Tags,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardsParam {
    Card(CardId),
    Tag(TagId),
}

impl Route {
    pub const DEFAULT: Self = Self::Review;

    const fn next(self) -> Self {
        match self {
            Self::Review => Self::Editor(None),
            Self::Editor(_) => Self::Cards(None),
            Self::Cards(_) => Self::Tags,
            Self::Tags => Self::Settings,
            Self::Settings => Self::Review,
        }
    }

    const fn prev(self) -> Self {
        match self {
            Self::Review => Self::Settings,
            Self::Editor(_) => Self::Review,
            Self::Cards(_) => Self::Editor(None),
            Self::Tags => Self::Cards(None),
            Self::Settings => Self::Tags,
        }
    }
}

enum PageState {
    Route,
    Search,
    Logs,
}

impl Pages {
    pub fn new(route: Route, settings: &Settings, db: &mut Database, markup: &mut Markup) -> Self {
        let mut pages = Self {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(),
            cards: CardsPage::new(),
            tags: TagsPage::new(db),
            settings: SettingsPage::new(&settings),
            search: SearchPage::new(),
            logs: LogsPage::new(),
            route: Route::Review,
            state: PageState::Route,
        };
        pages.call_on_enter(route, db, markup);
        pages
    }

    pub fn next(&mut self, db: &mut Database, markup: &mut Markup) {
        match self.state {
            PageState::Route => {
                self.set_route(self.route.next(), db, markup);
            }
            PageState::Search => {
                self.search.on_exit();
                self.state = PageState::Route;
            }
            PageState::Logs => {
                self.logs.on_exit();
                self.state = PageState::Route;
            }
        }
    }

    pub fn previous(&mut self, db: &mut Database, markup: &mut Markup) {
        match self.state {
            PageState::Route => {
                self.set_route(self.route.prev(), db, markup);
            }
            PageState::Search => {
                self.search.on_exit();
                self.state = PageState::Route;
            }
            PageState::Logs => {
                self.logs.on_exit();
                self.state = PageState::Route;
            }
        }
    }

    pub fn set_route(&mut self, route: Route, db: &mut Database, markup: &mut Markup) {
        match self.state {
            PageState::Route => {
                if self.route == route {
                    return;
                }

                self.call_on_exit(self.route);
                self.route = route;
                self.call_on_enter(route, db, markup);
            }
            PageState::Search => {
                self.search.on_exit();
                self.state = PageState::Route;

                if self.route != route {
                    self.call_on_exit(self.route);
                    self.route = route;
                    self.call_on_enter(route, db, markup);
                }
            }
            PageState::Logs => {
                self.logs.on_exit();
                self.state = PageState::Route;

                if self.route != route {
                    self.call_on_exit(self.route);
                    self.route = route;
                    self.call_on_enter(route, db, markup);
                }
            }
        }
    }

    pub fn toggle_search(&mut self, db: &mut Database) {
        match self.state {
            PageState::Route => {
                self.search.on_enter(db);
                self.state = PageState::Search;
            }
            PageState::Search => {
                self.search.on_exit();
                self.state = PageState::Route;
            }
            PageState::Logs => {
                self.logs.on_exit();
                self.search.on_enter(db);
                self.state = PageState::Search;
            }
        }
    }

    pub fn toggle_logs(&mut self) {
        match self.state {
            PageState::Route => {
                self.logs.on_enter();
                self.state = PageState::Logs;
            }
            PageState::Search => {
                self.search.on_exit();
                self.logs.on_enter();
                self.state = PageState::Logs;
            }
            PageState::Logs => {
                self.logs.on_exit();
                self.state = PageState::Route;
            }
        }
    }

    fn call_on_enter(&mut self, route: Route, db: &mut Database, markup: &mut Markup) {
        match route {
            Route::Review => self.review.on_enter(db, markup),
            Route::Editor(id) => self.editor.on_enter(id, db),
            Route::Cards(id) => self.cards.on_enter(db, id),
            Route::Tags => self.tags.on_enter(),
            Route::Settings => self.settings.on_enter(),
        };
    }

    fn call_on_exit(&mut self, route: Route) {
        match route {
            Route::Review => self.review.on_exit(),
            Route::Editor(_) => self.editor.on_exit(),
            Route::Cards(_) => self.cards.on_exit(),
            Route::Tags => self.tags.on_exit(),
            Route::Settings => self.settings.on_exit(),
        }
    }

    pub fn on_render(
        &mut self,
        render: AppRender,
        settings: &Settings,
        db: &mut Database,
        markup: &mut Markup,
        kitty: &mut KittyGraphics,
        shortcuts: &mut Shortcuts,
    ) {
        match self.state {
            PageState::Route => match self.route {
                Route::Review => {
                    self.review
                        .on_render(render, db, settings.colors(), markup, kitty, shortcuts)
                }
                Route::Editor(_) => {
                    self.editor
                        .on_render(render, db, settings.colors(), markup, kitty, shortcuts)
                }
                Route::Cards(_) => {
                    self.cards
                        .on_render(render, db, settings.colors(), markup, kitty, shortcuts)
                }
                Route::Tags => self.tags.on_render(render, settings.colors(), shortcuts),
                Route::Settings => self.settings.on_render(render, settings, shortcuts),
            },
            PageState::Search => {
                self.search
                    .on_render(render, settings.colors(), markup, kitty, shortcuts)
            }
            PageState::Logs => self.logs.on_render(render, settings.colors(), shortcuts),
        }
    }

    pub fn on_input(
        &mut self,
        input: AppInput,
        db: &mut Database,
        markup: &mut Markup,
        terminal: &mut Terminal,
        settings: &mut Settings,
    ) -> Action {
        match self.state {
            PageState::Route => match self.route {
                Route::Review => self.review.on_input(input, markup, db),
                Route::Editor(_) => self.editor.on_input(input, markup, db, terminal),
                Route::Cards(_) => self.cards.on_input(input, markup, db),
                Route::Tags => self.tags.on_input(input, db),
                Route::Settings => self.settings.on_input(input, settings),
            },
            PageState::Search => self.search.on_input(input, db, markup),
            PageState::Logs => self.logs.on_input(input),
        }
    }

    pub fn render_navigation(&self, area: Rect, buf: &mut Buffer, colors: &Colors) {
        const SPACING: &str = "   ";
        widgets::print_asciis_with_styles(
            area,
            buf,
            [
                (Route::Review, "Review", SPACING),
                (Route::Editor(None), "Editor", SPACING),
                (Route::Cards(None), "Cards", SPACING),
                (Route::Tags, "Tags", SPACING),
                (Route::Settings, "Settings", ""),
            ]
            .into_iter()
            .map(|(route, name, spacing)| {
                let is_current =
                    std::mem::discriminant(&route) == std::mem::discriminant(&self.route);
                let style = if is_current {
                    Style::new().fg(colors.primary).bold()
                } else {
                    Style::new()
                };
                ((name, style), (spacing, Style::new()))
            })
            .flat_map(|(a, b)| [a, b]),
            Some(widgets::Alignment::CenterHorizontal),
        );
    }

    pub const fn apply_settings(&mut self, settings: &Settings) {
        self.review
            .set_desired_retention(settings.desired_retention_as_fraction());
    }

    pub const fn is_logs_empty(&self) -> bool {
        self.logs.is_empty()
    }

    pub const fn logs_queue_len(&self) -> u32 {
        self.logs.queue_len()
    }

    pub fn enqueue_log(&mut self, log: Log) {
        self.logs.enqueue(log);
    }
}

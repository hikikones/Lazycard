use crossterm::event::{Event, KeyCode, KeyEventKind};
use layout::Flex;
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Borders, Paragraph},
    DefaultTerminal,
};

fn main() -> std::io::Result<()> {
    App::new().run(ratatui::init())?;
    ratatui::restore();

    Ok(())
}

struct App {
    running: bool,
    page: Page,
}

enum Page {
    Review,
    Cards,
}

impl Page {
    fn render<'a>(&self, body: Rect, buf: &mut Buffer) {
        match self {
            Page::Review => {
                Paragraph::new("review...").render(body, buf);
            }
            Page::Cards => {
                Paragraph::new("cards...").render(body, buf);
            }
        }
    }

    fn event(&self, event: Event) {
        //todo
    }

    const fn title(&self) -> &'static str {
        match self {
            Page::Review => "Review",
            Page::Cards => "Cards",
        }
    }

    const fn footer(&self) -> &'static str {
        match self {
            Page::Review => "review...",
            Page::Cards => "cards...",
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            running: true,
            page: Page::Review,
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            let event = crossterm::event::read()?;
            match event {
                Event::Key(key) => match key.kind {
                    KeyEventKind::Press => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => self.running = false,
                        KeyCode::Tab => match self.page {
                            Page::Review => self.page = Page::Cards,
                            Page::Cards => self.page = Page::Review,
                        },
                        _ => self.page.event(event),
                    },
                    _ => self.page.event(event),
                },
                _ => self.page.event(event),
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(area);
        let body = center_horizontal(body.inner(Margin::new(2, 2)), Constraint::Length(64));

        // Header
        Block::new()
            .title(
                Title::from(format!("  Lazycard: {}  ", self.page.title()).bold())
                    .alignment(Alignment::Center),
            )
            .borders(Borders::TOP)
            .render(header, buf);

        // Body
        self.page.render(body, buf);

        // Footer
        Line::from(format!("  ESC to quit  |  {}  ", self.page.footer()))
            .centered()
            .render(footer, buf);
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

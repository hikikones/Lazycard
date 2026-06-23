use ratatui::crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{CompletedFrame, DefaultTerminal, Frame};

pub struct Terminal(DefaultTerminal);

impl Terminal {
    pub fn init() -> std::io::Result<Self> {
        let terminal = ratatui::try_init()?;

        Ok(Self(terminal))
    }

    pub fn restore() -> std::io::Result<()> {
        ratatui::try_restore()
    }

    pub fn draw<F>(&mut self, render_callback: F) -> std::io::Result<CompletedFrame<'_>>
    where
        F: FnOnce(&mut Frame) -> std::io::Result<()>,
    {
        self.0.try_draw(render_callback)
    }

    pub fn temp_leave<T>(&mut self, f: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
        let mut stdout = std::io::stdout();

        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        let t = f();

        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        self.0.clear()?;

        t
    }
}

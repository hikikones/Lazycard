use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{CompletedFrame, DefaultTerminal, Frame};

pub struct Terminal {
    terminal: DefaultTerminal,
    clear: bool,
}

pub fn init() -> std::io::Result<Terminal> {
    let terminal = ratatui::try_init()?;

    Ok(Terminal {
        terminal,
        clear: false,
    })
}

pub fn restore() -> std::io::Result<()> {
    ratatui::try_restore()
}

impl Terminal {
    pub fn draw<F>(&mut self, render_callback: F) -> std::io::Result<CompletedFrame>
    where
        F: FnOnce(&mut Frame),
    {
        if self.clear {
            self.terminal.clear()?;
            self.clear = false;
        }

        self.terminal.try_draw(|frame| {
            render_callback(frame);
            std::io::Result::Ok(())
        })
    }

    pub fn temp_leave<T>(&mut self, f: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
        let mut stdout = std::io::stdout();

        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;

        let t = f();

        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        self.clear = true;

        t
    }
}

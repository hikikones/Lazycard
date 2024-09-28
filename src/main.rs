use crossterm::event::{Event, KeyCode, KeyEventKind};
use layout::Flex;
use ratatui::{prelude::*, widgets::Paragraph, DefaultTerminal};

mod app;
mod navigation;

use app::*;
use navigation::*;

fn main() -> std::io::Result<()> {
    App::new().run(ratatui::init())?;
    ratatui::restore();

    Ok(())
}

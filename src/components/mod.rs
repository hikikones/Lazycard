mod button;
mod dropdown;
mod icon;
mod markdown_editor;
mod markdown_view;
mod tags;

pub use button::*;
pub use dropdown::*;
pub use icon::*;
pub use markdown_editor::*;
pub use markdown_view::*;
pub use tags::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl TextSize {
    pub const fn var(&self) -> &str {
        match self {
            TextSize::Small => "var(--text-small)",
            TextSize::Medium => "var(--text-medium)",
            TextSize::Large => "var(--text-large)",
        }
    }
}

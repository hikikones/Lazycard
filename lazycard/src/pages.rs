mod cards;
mod editor;
mod editor2;
mod logs;
mod review;

pub use cards::*;
pub use editor::*;
pub use editor2::*;
pub use logs::*;
pub use review::*;

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage2,
    pub cards: CardsPage,
    pub logs: LogsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<database::CardId>),
    Cards,
}

impl Route {
    pub const fn next(self) -> Self {
        match self {
            Self::Review => Self::Editor(None),
            Self::Editor(_) => Self::Cards,
            Self::Cards => Self::Review,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Review => Self::Cards,
            Self::Editor(_) => Self::Review,
            Self::Cards => Self::Editor(None),
        }
    }
}

impl Default for Route {
    fn default() -> Self {
        Self::Review
    }
}

mod cards;
mod editor;
mod logs;
mod review;
mod search;
mod tags;

pub use cards::*;
pub use editor::*;
pub use logs::*;
pub use review::*;
pub use search::*;
pub use tags::*;

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage,
    pub cards: CardsPage,
    pub tags: TagsPage,
    pub search: SearchPage,
    pub logs: LogsPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<database::CardId>),
    Cards(Option<database::CardId>),
    Tags,
}

impl Route {
    pub const fn next(self) -> Self {
        match self {
            Self::Review => Self::Editor(None),
            Self::Editor(_) => Self::Cards(None),
            Self::Cards(_) => Self::Tags,
            Self::Tags => Self::Review,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Review => Self::Tags,
            Self::Editor(_) => Self::Review,
            Self::Cards(_) => Self::Editor(None),
            Self::Tags => Self::Cards(None),
        }
    }
}

impl Default for Route {
    fn default() -> Self {
        Self::Review
    }
}

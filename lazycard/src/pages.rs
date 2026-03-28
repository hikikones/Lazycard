mod cards;
mod editor;
mod review;

pub use cards::*;
pub use editor::*;
pub use review::*;

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage,
    pub cards: CardsPage,
}

impl Pages {
    pub fn new(colors: &crate::settings::Colors) -> Self {
        Self {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(colors),
            cards: CardsPage::new(colors),
        }
    }
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

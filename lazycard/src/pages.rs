mod cards;
mod editor;
mod review;

pub use cards::*;
pub use editor::*;
pub use review::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Review,
    Editor(Option<database::CardId>),
    Cards,
}

pub struct Pages {
    pub review: ReviewPage,
    pub editor: CardEditorPage,
    pub cards: CardsPage,
}

impl Pages {
    pub fn new(external_editor: bool, colors: &crate::settings::Colors) -> Self {
        Self {
            review: ReviewPage::new(),
            editor: CardEditorPage::new(external_editor, colors),
            cards: CardsPage::new(colors),
        }
    }
}

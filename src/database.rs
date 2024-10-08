use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Database {
    btree: BTreeMap<CardId, Card>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            btree: Default::default(),
        }
    }

    pub fn add(&mut self, card: Card) {
        let last_id = self.keys().last().copied().unwrap_or_default().0;
        self.insert(CardId(last_id + 1), card);
    }
}

impl std::ops::Deref for Database {
    type Target = BTreeMap<CardId, Card>;

    fn deref(&self) -> &Self::Target {
        &self.btree
    }
}

impl std::ops::DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.btree
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u64);

#[derive(Debug, Default, Clone)]
pub struct Card {
    pub content: String,
    pub kind: CardKind,
}

impl Card {
    pub fn new(kind: CardKind, content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            kind,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum CardKind {
    #[default]
    Text,
    Typst,
}

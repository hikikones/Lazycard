use std::path::PathBuf;

use crate::{Card, CardId, Scheduler, Storage};

pub struct Database {
    path: PathBuf,
    storage: Storage,
    scheduler: Scheduler,
    is_dirty: bool,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let storage = if let Ok(true) = path.try_exists() {
            Storage::read(&path)?
        } else {
            let storage = Storage::default();
            storage.write(&path)?;
            storage
        };

        Ok(Self {
            path,
            storage,
            scheduler: Scheduler::new(),
            is_dirty: false,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.storage.cards.is_empty()
    }

    pub fn get(&self, id: CardId) -> Option<&Card> {
        self.storage.cards.get(&id)
    }

    pub fn add(&mut self, card: Card) {
        let last_id = self.last_id();
        self.storage.cards.insert(CardId(last_id.0 + 1), card);
        self.is_dirty = true;
    }

    pub fn update(&mut self, id: CardId, func: impl FnOnce(&mut Card)) {
        if let Some(card) = self.storage.cards.get_mut(&id) {
            self.is_dirty = true;
            func(card);
        }
    }

    pub fn remove(&mut self, id: CardId) -> Option<Card> {
        if let Some(card) = self.storage.cards.remove(&id) {
            self.is_dirty = true;
            return Some(card);
        }
        None
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (CardId, &Card)> + DoubleEndedIterator {
        self.storage.cards.iter().map(|(id, card)| (*id, card))
    }

    pub fn schedule(&mut self, id: CardId, success: bool) {
        if let Some(card) = self.storage.cards.get_mut(&id) {
            self.scheduler.schedule(card, success);
            self.is_dirty = true;
        }
    }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_dirty {
            let res = self.storage.write(&self.path);
            self.is_dirty = res.is_err();
            return res;
        }
        Ok(())
    }

    fn last_id(&self) -> CardId {
        self.storage
            .cards
            .keys()
            .last()
            .copied()
            .unwrap_or_default()
    }
}

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::utils::{MatchScore, Matcher};

pub struct Database {
    path: PathBuf,
    storage: Storage,
    scheduler: Scheduler,
    is_dirty: bool,
}

impl Database {
    pub fn new(path: PathBuf, desired_retention: f32) -> Result<Self, Box<dyn std::error::Error>> {
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
            scheduler: Scheduler::new(desired_retention),
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
            card.schedule(success, &mut self.scheduler);
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CardId(u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub content: String,
    pub creation_time: UnixTime,
    pub last_review_time: Option<UnixTime>,
    pub review_interval: f32,
    pub review_stability: f32,
    pub review_difficulty: f32,
    pub archived: bool,
}

impl Card {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            creation_time: UnixTime::now(),
            last_review_time: None,
            review_interval: 0.0,
            review_stability: 0.0,
            review_difficulty: 0.0,
            archived: false,
        }
    }

    fn is_due(&self, now: UnixTime) -> bool {
        let review_time = self.last_review_time.unwrap_or(self.creation_time);
        let due_time = review_time.add_days(self.review_interval);
        due_time <= now
    }

    fn schedule(&mut self, success: bool, scheduler: &mut Scheduler) {
        let review_state = scheduler.schedule(self, success);
        self.last_review_time = review_state.time.into();
        self.review_interval = review_state.interval;
        self.review_stability = review_state.stability;
        self.review_difficulty = review_state.difficulty;
    }
}

#[derive(Serialize, Deserialize)]
struct Storage {
    version: u32,
    cards: BTreeMap<CardId, Card>,
}

impl Storage {
    const VERSION: u32 = 1;

    fn read(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;

        #[derive(Deserialize)]
        struct V {
            version: u32,
        }

        let ron_options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::UNWRAP_NEWTYPES);
        let V { version } = ron_options.from_str(&file_content)?;

        match version {
            Self::VERSION => ron_options.from_str(&file_content).map_err(|e| e.into()),
            _ => Err("unknown version found in database".into()),
        }
    }

    fn write(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let ron_options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::UNWRAP_NEWTYPES);
        let ron_string = ron_options.to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, ron_string).map_err(|e| e.into())
    }
}

impl Default for Storage {
    fn default() -> Self {
        let mut cards = BTreeMap::new();
        add_test_data(&mut cards);

        Self {
            version: Self::VERSION,
            cards,
        }
    }
}

struct Scheduler {
    fsrs: fsrs::FSRS,
    desired_retention: f32,
}

impl Scheduler {
    fn new(desired_retention: f32) -> Self {
        Self {
            fsrs: fsrs::FSRS::new(Some(&fsrs::DEFAULT_PARAMETERS)).unwrap(),
            desired_retention,
        }
    }

    fn schedule(&mut self, card: &Card, success: bool) -> ReviewState {
        let current_memory_state = if card.review_stability == 0.0 || card.review_difficulty == 0.0
        {
            None
        } else {
            Some(fsrs::MemoryState {
                stability: card.review_stability,
                difficulty: card.review_difficulty,
            })
        };
        let now = UnixTime::now();
        let days_since_last_review = if let Some(last_review_time) = card.last_review_time {
            last_review_time.days_since(now)
        } else {
            0
        };
        let next_states = self
            .fsrs
            .next_states(
                current_memory_state,
                self.desired_retention,
                days_since_last_review,
            )
            .unwrap();
        let state = if success {
            next_states.good
        } else {
            next_states.again
        };

        ReviewState {
            time: now,
            interval: state.interval,
            stability: state.memory.stability,
            difficulty: state.memory.difficulty,
        }
    }
}

struct ReviewState {
    time: UnixTime,
    interval: f32,
    stability: f32,
    difficulty: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixTime(u64);

impl UnixTime {
    const SECONDS_PER_DAY: u64 = 86400;

    fn now() -> Self {
        let secs_since_unix_epoch = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self(secs_since_unix_epoch)
    }

    const fn days_since(self, other: Self) -> u32 {
        (self.0.abs_diff(other.0) / Self::SECONDS_PER_DAY) as u32
    }

    const fn add_days(self, days: f32) -> Self {
        let days_in_secs = days * Self::SECONDS_PER_DAY as f32;
        Self(self.0 + days_in_secs as u64)
    }
}

pub trait CardsIterExt<'a> {
    fn due(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn active(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)>;
    fn search(
        self,
        pattern: &str,
        matcher: &'a mut Matcher,
    ) -> impl Iterator<Item = (CardId, &'a Card, MatchScore)>;
}

impl<'a, I> CardsIterExt<'a> for I
where
    I: Iterator<Item = (CardId, &'a Card)>,
{
    fn due(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        let now = UnixTime::now();
        self.filter(move |(_, card)| !card.archived && card.is_due(now))
    }

    fn active(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        self.filter(|(_, card)| !card.archived)
    }

    fn _archived(self) -> impl Iterator<Item = (CardId, &'a Card)> {
        self.filter(|(_, card)| card.archived)
    }

    fn search(
        self,
        pattern: &str,
        matcher: &'a mut Matcher,
    ) -> impl Iterator<Item = (CardId, &'a Card, MatchScore)> {
        matcher.update(pattern);
        self.filter_map(|(id, card)| {
            matcher
                .score(card.content.as_str())
                .map(|score| (id, card, MatchScore::new(score)))
        })
    }
}

fn add_test_data(cards: &mut BTreeMap<CardId, Card>) {
    cards.insert(
        CardId(1),
        Card::new(
            r#"
# this is a comment
left paragraph with *bold*, _italic_ and maybe `verbatim text` that *should wrap* when line becomes _tooooooooo_ long...

- item 1
- item 2 with lots of text that can wrap to next line but also keeping the indent so it looks nice ohhh yeah
- item 3

> right paragraph

| center paragraph

```rust
fn main() {
    println!("Hello, world!");
}
```

---

Lorem ipsum dolor sit amet,
consectetur adipiscing elit.
Donec fermentum ipsum nec sagittis feugiat.
Curabitur pulvinar et orci luctus faucibus.
In erat justo, placerat et risus quis, cursus elementum mi.

---

another one

---

and another one
"#,
        ),
    );

    cards.insert(
        CardId(2),
        Card::new(
            r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec fermentum ipsum nec sagittis feugiat. Curabitur pulvinar et orci luctus faucibus. In erat justo, placerat et risus quis, cursus elementum mi. Donec non leo est. Etiam consectetur, lectus nec auctor sodales, velit arcu tincidunt justo, mattis dictum odio libero ac enim. Morbi maximus, tortor id ornare tristique, purus mi pulvinar urna, vel accumsan ante nunc vel urna. Maecenas ligula elit, tempor eget augue ut, auctor blandit ligula. Mauris maximus condimentum aliquam. Nam purus enim, ornare ut suscipit et, bibendum a ex. Donec euismod velit quis nisi convallis, rhoncus lacinia sapien aliquam. Ut dolor magna, imperdiet eu arcu vitae, consectetur feugiat velit. Donec eu dolor eu tortor rutrum egestas quis in orci. Maecenas pulvinar, massa eget fringilla convallis, dolor tellus luctus elit, in tempus nibh dolor at dolor. Nulla viverra et justo sit amet rhoncus. Fusce porttitor odio in lectus ullamcorper vehicula.

Nullam pharetra augue non leo maximus convallis a sit amet risus. Integer congue laoreet efficitur. Interdum et malesuada fames ac ante ipsum primis in faucibus. Quisque vestibulum, nisl vitae porttitor maximus, nulla mauris cursus ligula, vel consectetur magna dolor ac nulla. Integer at pharetra nisi, quis congue ante. Nam et varius purus. Maecenas placerat sapien at ante cursus, at volutpat felis facilisis. Fusce id metus vel urna molestie tincidunt. Phasellus sed dapibus ligula.

Vivamus nec dui pellentesque, interdum dui vitae, egestas ex. Morbi malesuada porta vehicula. Donec dapibus mattis arcu, vel bibendum diam lobortis et. Curabitur sit amet felis aliquet enim mollis semper. Mauris diam orci, rutrum sed neque vel, malesuada sollicitudin augue. Integer vehicula dolor consectetur tincidunt imperdiet. Donec at dui urna. Duis et turpis in diam ornare volutpat.

Mauris suscipit imperdiet mi et semper. Nam nec lorem sagittis, lobortis ligula convallis, hendrerit tortor. Morbi dapibus magna ut sollicitudin placerat. Etiam sodales varius ante convallis hendrerit. Duis varius quam et tincidunt vestibulum. Fusce facilisis elit eu ligula consequat, dignissim elementum diam aliquet. Maecenas ipsum tellus, condimentum sit amet ultrices et, semper id nibh. Fusce varius porttitor dui vitae fermentum. Nam placerat, nunc eget tempus egestas, enim nisl scelerisque erat, in venenatis leo leo a leo. Nam quis nibh vitae turpis efficitur lobortis id a metus.

In aliquet dui sapien, ut semper elit sodales sed. Proin quis libero luctus libero scelerisque ornare eget id mi. Cras accumsan arcu ut ante pharetra fringilla. Sed feugiat placerat dolor, et feugiat lorem iaculis id. Cras interdum est nec elit molestie, sed aliquam quam hendrerit. Proin sit amet pellentesque enim. Nam bibendum, mauris vel eleifend ultricies, orci sapien hendrerit sem, a tristique lectus nulla a ligula.
"#,
        ),
    );
    cards.insert(CardId(3), Card::new("👻 oijwqwu qwdiowhq  i hio h qiowhqwheqw👻👻 wwq qiuwhdidwh👻👻👻❤️\n\nauhui ❤️awudhia\n🧑‍🌾❤️👨‍🦰jfpkw huiw wjwioj ijf weoijwioejfiowejfiowjfiowej\n\nthis\tis\ta\tparagraph\twith\ttabs\n\n```rust\nfn main() {\n\tprintln!(\"Hello, world!\");\n}\n```"));
}

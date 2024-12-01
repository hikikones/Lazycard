use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Database {
    btree: BTreeMap<CardId, Card>,
    #[serde(skip)]
    _path: PathBuf,
    #[serde(skip)]
    matcher: Matcher,
    #[serde(skip)]
    scheduler: Scheduler,
}

impl Database {
    pub fn new(_path: PathBuf, desired_retention: f32) -> Self {
        // todo: use path

        let mut btree = BTreeMap::new();
        add_test_data(&mut btree);

        Self {
            btree,
            _path,
            matcher: Matcher::new(),
            scheduler: Scheduler::new(desired_retention),
        }
    }

    pub fn add(&mut self, card: Card) {
        let last_id = self.keys().last().copied().unwrap_or_default().0;
        self.insert(CardId(last_id + 1), card);
    }

    pub fn due(&self) -> impl Iterator<Item = (CardId, &Card)> {
        let now = UnixTime::now();
        self.btree
            .iter()
            .filter(move |(_, card)| {
                let review_time = card.last_review.unwrap_or(card.creation_time);
                let due_time = review_time.add_days(card.interval);
                due_time <= now
            })
            .map(|(id, card)| (*id, card))
    }

    pub fn search(&mut self, pattern: &str) -> impl Iterator<Item = (CardId, &Card, MatchScore)> {
        self.matcher.pattern(pattern);
        self.btree.iter().filter_map(|(id, card)| {
            self.matcher
                .score(card.content.as_str())
                .map(|score| (*id, card, MatchScore(score)))
        })
    }

    pub fn schedule(&mut self, id: CardId, success: bool) {
        let card = self.btree.get_mut(&id).unwrap();
        let next_review_state = self.scheduler.schedule(card, success);
        card.last_review = Some(UnixTime::now());
        card.interval = next_review_state.interval;
        card.stability = next_review_state.stability;
        card.difficulty = next_review_state.difficulty;
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

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct CardId(u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    content: String,
    creation_time: UnixTime,
    last_review: Option<UnixTime>,
    interval: f32,
    stability: f32,
    difficulty: f32,
}

impl Card {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            creation_time: UnixTime::now(),
            last_review: None,
            interval: 0.0,
            stability: 0.0,
            difficulty: 0.0,
        }
    }

    pub fn get_content(&self) -> &str {
        self.content.as_str()
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
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
        let current_memory_state = if card.stability == 0.0 || card.difficulty == 0.0 {
            None
        } else {
            Some(fsrs::MemoryState {
                stability: card.stability,
                difficulty: card.difficulty,
            })
        };
        let days_since_last_review = if let Some(last_review_time) = card.last_review {
            last_review_time.days_since(UnixTime::now())
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
        let next_review_state = if success {
            next_states.good
        } else {
            next_states.again
        };

        ReviewState {
            interval: next_review_state.interval,
            stability: next_review_state.memory.stability,
            difficulty: next_review_state.memory.difficulty,
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(0.8)
    }
}

struct ReviewState {
    interval: f32,
    stability: f32,
    difficulty: f32,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
struct UnixTime(u64);

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatchScore(u32);

struct Matcher {
    matcher: nucleo_matcher::Matcher,
    pattern: nucleo_matcher::pattern::Pattern,
    buffer: Vec<char>,
}

impl Matcher {
    fn new() -> Self {
        Self {
            matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
            pattern: nucleo_matcher::pattern::Pattern::new(
                "",
                nucleo_matcher::pattern::CaseMatching::Smart,
                nucleo_matcher::pattern::Normalization::Smart,
                nucleo_matcher::pattern::AtomKind::Fuzzy,
            ),
            buffer: Vec::new(),
        }
    }

    fn pattern(&mut self, pattern: &str) {
        self.pattern.reparse(
            pattern,
            nucleo_matcher::pattern::CaseMatching::Smart,
            nucleo_matcher::pattern::Normalization::Smart,
        );
    }

    fn score(&mut self, haystack: &str) -> Option<u32> {
        self.pattern.score(
            nucleo_matcher::Utf32Str::new(haystack, &mut self.buffer),
            &mut self.matcher,
        )
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}

fn add_test_data(db: &mut BTreeMap<CardId, Card>) {
    db.insert(
        CardId(1),
        Card::new(
            r#"
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

    db.insert(
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
    db.insert(CardId(3), Card::new("👻 oijwqwu qwdiowhq  i hio h qiowhqwheqw👻👻 wwq qiuwhdidwh👻👻👻❤️\n\nauhui ❤️awudhia\n🧑‍🌾❤️👨‍🦰jfpkw huiw wjwioj ijf weoijwioejfiowejfiowjfiowej\n\nthis\tis\ta\tparagraph\twith\ttabs\n\n```rust\nfn main() {\n\tprintln!(\"Hello, world!\");\n}\n```"));
}

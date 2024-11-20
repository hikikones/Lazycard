use std::collections::BTreeMap;

pub struct Database {
    btree: BTreeMap<CardId, Card>,
    matcher: Matcher,
}

impl Database {
    pub fn new() -> Self {
        let mut btree = BTreeMap::new();
        add_test_data(&mut btree);

        Self {
            btree,
            matcher: Matcher::new(),
        }
    }

    pub fn add(&mut self, card: Card) {
        let last_id = self.keys().last().copied().unwrap_or_default().0;
        self.insert(CardId(last_id + 1), card);
    }

    pub fn search(&mut self, pattern: &str) -> impl Iterator<Item = (CardId, &Card, MatchScore)> {
        self.matcher.pattern(pattern);
        self.btree.iter().filter_map(|(id, card)| {
            self.matcher
                .score(&card.0)
                .map(|score| (*id, card, MatchScore(score)))
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u64);

#[derive(Debug, Default, Clone)]
pub struct Card(pub String);

impl Card {
    pub fn new(content: impl Into<String>) -> Self {
        Self(content.into())
    }
}

pub struct MatchScore(pub u32);

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

fn add_test_data(db: &mut BTreeMap<CardId, Card>) {
    db.insert(
        CardId(1),
        Card::new(
            r#"
left paragraph with **bold** and __italic__ text that **should wrap** when line becomes __tooooooooo__ long...

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
    db.insert(CardId(3), Card::new("👻 oijwqwu qwdiowhq  i hio h qiowhqwheqw👻👻 wwq qiuwhdidwh👻👻👻\n\nthis\tis\ta\tparagraph\twith\ttabs\n\n```rust\nfn main() {\n\tprintln!(\"Hello, world!\");\n}\n```"));
}

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Card, CardId};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Storage {
    version: u32,
    pub(crate) cards: BTreeMap<CardId, Card>,
}

impl Storage {
    const VERSION: u32 = 1;

    pub(crate) fn read(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub(crate) fn write(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

/// ANSI writer and parser for most Select Graphic Rendition (SGR) attributes.
use std::fmt::Write;

const ANSI_SEQUENCE_START: &str = "\x1b[";
const ANSI_SEQUENCE_END: char = 'm';

#[derive(Debug)]
pub enum AnsiTag {
    // Reset
    Reset,

    // Style on
    Bold,
    Faint,
    Italic,
    Underline,
    SlowBlink,
    RapidBlink,
    Reverse,
    Conceal,
    CrossedOut,
    Framed,
    Encircled,
    Overlined,

    // Style off
    NotBold,
    NotItalic,
    NotUnderline,
    NotBlink,
    NotReverse,
    Reveal,
    NotCrossedOut,
    NotFramedOrEncircled,
    NotOverlined,

    // Foreground colors
    FgBlack,
    FgRed,
    FgGreen,
    FgYellow,
    FgBlue,
    FgMagenta,
    FgCyan,
    FgWhite,
    FgBrightBlack,
    FgBrightRed,
    FgBrightGreen,
    FgBrightYellow,
    FgBrightBlue,
    FgBrightMagenta,
    FgBrightCyan,
    FgBrightWhite,
    FgDefault,

    // Background colors
    BgBlack,
    BgRed,
    BgGreen,
    BgYellow,
    BgBlue,
    BgMagenta,
    BgCyan,
    BgWhite,
    BgBrightBlack,
    BgBrightRed,
    BgBrightGreen,
    BgBrightYellow,
    BgBrightBlue,
    BgBrightMagenta,
    BgBrightCyan,
    BgBrightWhite,
    BgDefault,

    // Extended colors
    Fg256(u8),
    Bg256(u8),
    FgTrueColor(u8, u8, u8),
    BgTrueColor(u8, u8, u8),
}

pub struct AnsiWriter {
    inner: String,
}

impl AnsiWriter {
    pub const fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    pub const fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub const fn inner(&self) -> &String {
        &self.inner
    }

    pub const fn inner_mut(&mut self) -> &mut String {
        &mut self.inner
    }

    pub fn push_char(&mut self, c: char) {
        self.inner.push(c);
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }

    pub fn push_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        use std::fmt::Write;
        let _ = self.inner.write_fmt(args);
    }

    pub fn push_tag(&mut self, tag: AnsiTag) {
        self.inner.push_str(ANSI_SEQUENCE_START);
        self.write_tag(tag);
        self.inner.push(ANSI_SEQUENCE_END);
    }

    pub fn extend<'a>(&mut self, iter: impl IntoIterator<Item = &'a str>) {
        self.inner.extend(iter);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    fn write_tag(&mut self, tag: AnsiTag) {
        match tag {
            AnsiTag::Reset => self.inner.push('0'),

            AnsiTag::Bold => self.inner.push('1'),
            AnsiTag::Faint => self.inner.push('2'),
            AnsiTag::Italic => self.inner.push('3'),
            AnsiTag::Underline => self.inner.push('4'),
            AnsiTag::SlowBlink => self.inner.push('5'),
            AnsiTag::RapidBlink => self.inner.push('6'),
            AnsiTag::Reverse => self.inner.push('7'),
            AnsiTag::Conceal => self.inner.push('8'),
            AnsiTag::CrossedOut => self.inner.push('9'),
            AnsiTag::Framed => self.inner.push_str("51"),
            AnsiTag::Encircled => self.inner.push_str("52"),
            AnsiTag::Overlined => self.inner.push_str("53"),

            AnsiTag::NotBold => self.inner.push_str("22"),
            AnsiTag::NotItalic => self.inner.push_str("23"),
            AnsiTag::NotUnderline => self.inner.push_str("24"),
            AnsiTag::NotBlink => self.inner.push_str("25"),
            AnsiTag::NotReverse => self.inner.push_str("27"),
            AnsiTag::Reveal => self.inner.push_str("28"),
            AnsiTag::NotCrossedOut => self.inner.push_str("29"),
            AnsiTag::NotFramedOrEncircled => self.inner.push_str("54"),
            AnsiTag::NotOverlined => self.inner.push_str("55"),

            AnsiTag::FgBlack => self.inner.push_str("30"),
            AnsiTag::FgRed => self.inner.push_str("31"),
            AnsiTag::FgGreen => self.inner.push_str("32"),
            AnsiTag::FgYellow => self.inner.push_str("33"),
            AnsiTag::FgBlue => self.inner.push_str("34"),
            AnsiTag::FgMagenta => self.inner.push_str("35"),
            AnsiTag::FgCyan => self.inner.push_str("36"),
            AnsiTag::FgWhite => self.inner.push_str("37"),

            AnsiTag::FgBrightBlack => self.inner.push_str("90"),
            AnsiTag::FgBrightRed => self.inner.push_str("91"),
            AnsiTag::FgBrightGreen => self.inner.push_str("92"),
            AnsiTag::FgBrightYellow => self.inner.push_str("93"),
            AnsiTag::FgBrightBlue => self.inner.push_str("94"),
            AnsiTag::FgBrightMagenta => self.inner.push_str("95"),
            AnsiTag::FgBrightCyan => self.inner.push_str("96"),
            AnsiTag::FgBrightWhite => self.inner.push_str("97"),

            AnsiTag::FgDefault => self.inner.push_str("39"),

            AnsiTag::BgBlack => self.inner.push_str("40"),
            AnsiTag::BgRed => self.inner.push_str("41"),
            AnsiTag::BgGreen => self.inner.push_str("42"),
            AnsiTag::BgYellow => self.inner.push_str("43"),
            AnsiTag::BgBlue => self.inner.push_str("44"),
            AnsiTag::BgMagenta => self.inner.push_str("45"),
            AnsiTag::BgCyan => self.inner.push_str("46"),
            AnsiTag::BgWhite => self.inner.push_str("47"),

            AnsiTag::BgBrightBlack => self.inner.push_str("100"),
            AnsiTag::BgBrightRed => self.inner.push_str("101"),
            AnsiTag::BgBrightGreen => self.inner.push_str("102"),
            AnsiTag::BgBrightYellow => self.inner.push_str("103"),
            AnsiTag::BgBrightBlue => self.inner.push_str("104"),
            AnsiTag::BgBrightMagenta => self.inner.push_str("105"),
            AnsiTag::BgBrightCyan => self.inner.push_str("106"),
            AnsiTag::BgBrightWhite => self.inner.push_str("107"),

            AnsiTag::BgDefault => self.inner.push_str("49"),

            AnsiTag::Fg256(n) => {
                let _ = write!(self.inner, "38;5;{}", n);
            }
            AnsiTag::Bg256(n) => {
                let _ = write!(self.inner, "48;5;{}", n);
            }
            AnsiTag::FgTrueColor(r, g, b) => {
                let _ = write!(self.inner, "38;2;{};{};{}", r, g, b);
            }
            AnsiTag::BgTrueColor(r, g, b) => {
                let _ = write!(self.inner, "48;2;{};{};{}", r, g, b);
            }
        }
    }
}

#[derive(Debug)]
pub enum AnsiEvent<'a> {
    Text(&'a str),
    Tag(AnsiTag),
}

pub struct AnsiParser<'a> {
    input: &'a str,
    start: usize,
    text_start: usize,
}

impl<'a> AnsiParser<'a> {
    pub const fn new(input: &'a str) -> Self {
        Self {
            input,
            start: 0,
            text_start: 0,
        }
    }
}

impl<'a> Iterator for AnsiParser<'a> {
    type Item = AnsiEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.input.len() {
            return None;
        }

        while let Some(i) = self.input[self.start..].find(ANSI_SEQUENCE_START) {
            let ansi_start = self.start + i;

            // Return text before we parse ansi code
            if self.text_start < ansi_start {
                let text = &self.input[self.text_start..ansi_start];
                self.start = ansi_start;
                self.text_start = ansi_start;
                return Some(AnsiEvent::Text(text));
            }

            // Parse ansi code
            const MAX_CODE_LEN: usize = 16;
            let code_start = ansi_start + ANSI_SEQUENCE_START.len();
            let code_end = (code_start + MAX_CODE_LEN + 1).min(self.input.len());
            let Some(end) = self.input[code_start..code_end].find(ANSI_SEQUENCE_END) else {
                self.start = code_start;
                continue;
            };

            let end = code_start + end;
            let code = &self.input[code_start..end];
            if code.contains(";") {
                // Extended colors
                let mut split = code.split(";");
                let count = split.clone().count();
                match count {
                    3 => {
                        // Indexed color
                        match (
                            split.next(),
                            split.next(),
                            split.next().map(|n| n.parse::<u8>()),
                        ) {
                            (Some("38"), Some("5"), Some(Ok(index))) => {
                                self.start = end + 1;
                                self.text_start = end + 1;
                                return Some(AnsiEvent::Tag(AnsiTag::Fg256(index)));
                            }
                            (Some("48"), Some("5"), Some(Ok(index))) => {
                                self.start = end + 1;
                                self.text_start = end + 1;
                                return Some(AnsiEvent::Tag(AnsiTag::Bg256(index)));
                            }
                            _ => {
                                self.start = code_start;
                                continue;
                            }
                        }
                    }
                    5 => {
                        // True color
                        match (
                            split.next(),
                            split.next(),
                            split.next().map(|n| n.parse::<u8>()),
                            split.next().map(|n| n.parse::<u8>()),
                            split.next().map(|n| n.parse::<u8>()),
                        ) {
                            (Some("38"), Some("2"), Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) => {
                                self.start = end + 1;
                                self.text_start = end + 1;
                                return Some(AnsiEvent::Tag(AnsiTag::FgTrueColor(r, g, b)));
                            }
                            (Some("48"), Some("2"), Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) => {
                                self.start = end + 1;
                                self.text_start = end + 1;
                                return Some(AnsiEvent::Tag(AnsiTag::BgTrueColor(r, g, b)));
                            }
                            _ => {
                                self.start = code_start;
                                continue;
                            }
                        }
                    }
                    _ => {
                        self.start = code_start;
                        continue;
                    }
                }
            } else {
                // Single code
                let Ok(num) = code.parse::<u8>() else {
                    self.start = code_start;
                    continue;
                };

                let tag = match num {
                    0 => Some(AnsiTag::Reset),

                    1 => Some(AnsiTag::Bold),
                    2 => Some(AnsiTag::Faint),
                    3 => Some(AnsiTag::Italic),
                    4 => Some(AnsiTag::Underline),
                    5 => Some(AnsiTag::SlowBlink),
                    6 => Some(AnsiTag::RapidBlink),
                    7 => Some(AnsiTag::Reverse),
                    8 => Some(AnsiTag::Conceal),
                    9 => Some(AnsiTag::CrossedOut),

                    22 => Some(AnsiTag::NotBold),
                    23 => Some(AnsiTag::NotItalic),
                    24 => Some(AnsiTag::NotUnderline),
                    25 => Some(AnsiTag::NotBlink),
                    27 => Some(AnsiTag::NotReverse),
                    28 => Some(AnsiTag::Reveal),
                    29 => Some(AnsiTag::NotCrossedOut),

                    30 => Some(AnsiTag::FgBlack),
                    31 => Some(AnsiTag::FgRed),
                    32 => Some(AnsiTag::FgGreen),
                    33 => Some(AnsiTag::FgYellow),
                    34 => Some(AnsiTag::FgBlue),
                    35 => Some(AnsiTag::FgMagenta),
                    36 => Some(AnsiTag::FgCyan),
                    37 => Some(AnsiTag::FgWhite),

                    39 => Some(AnsiTag::FgDefault),

                    40 => Some(AnsiTag::BgBlack),
                    41 => Some(AnsiTag::BgRed),
                    42 => Some(AnsiTag::BgGreen),
                    43 => Some(AnsiTag::BgYellow),
                    44 => Some(AnsiTag::BgBlue),
                    45 => Some(AnsiTag::BgMagenta),
                    46 => Some(AnsiTag::BgCyan),
                    47 => Some(AnsiTag::BgWhite),

                    49 => Some(AnsiTag::BgDefault),

                    51 => Some(AnsiTag::Framed),
                    52 => Some(AnsiTag::Encircled),
                    53 => Some(AnsiTag::Overlined),
                    54 => Some(AnsiTag::NotFramedOrEncircled),
                    55 => Some(AnsiTag::NotOverlined),

                    90 => Some(AnsiTag::FgBrightBlack),
                    91 => Some(AnsiTag::FgBrightRed),
                    92 => Some(AnsiTag::FgBrightGreen),
                    93 => Some(AnsiTag::FgBrightYellow),
                    94 => Some(AnsiTag::FgBrightBlue),
                    95 => Some(AnsiTag::FgBrightMagenta),
                    96 => Some(AnsiTag::FgBrightCyan),
                    97 => Some(AnsiTag::FgBrightWhite),

                    100 => Some(AnsiTag::BgBrightBlack),
                    101 => Some(AnsiTag::BgBrightRed),
                    102 => Some(AnsiTag::BgBrightGreen),
                    103 => Some(AnsiTag::BgBrightYellow),
                    104 => Some(AnsiTag::BgBrightBlue),
                    105 => Some(AnsiTag::BgBrightMagenta),
                    106 => Some(AnsiTag::BgBrightCyan),
                    107 => Some(AnsiTag::BgBrightWhite),

                    _ => None,
                };

                match tag {
                    Some(tag) => {
                        self.start = end + 1;
                        self.text_start = end + 1;
                        return Some(AnsiEvent::Tag(tag));
                    }
                    None => {
                        self.start = code_start;
                        continue;
                    }
                }
            }
        }

        self.start = self.input.len();

        let remaining = &self.input[self.text_start..];
        if remaining.is_empty() {
            None
        } else {
            Some(AnsiEvent::Text(remaining))
        }
    }
}

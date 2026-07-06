/// ANSI writer and parser for most Select Graphic Rendition (SGR) attributes.
use std::{fmt::Write, ops::Range, str::CharIndices};

use ratatui::style::{Color, Modifier, Style};

const ANSI_START: char = '\x1b';
const ANSI_START2: char = '[';
const ANSI_END: char = 'm';

#[derive(Debug, Clone, Copy)]
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
    // Framed,
    // Encircled,
    // Overlined,

    // Style off
    NotBold,
    NotItalic,
    NotUnderline,
    NotBlink,
    NotReverse,
    Reveal,
    NotCrossedOut,
    // NotFramedOrEncircled,
    // NotOverlined,

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

impl AnsiTag {
    const fn from_u8(value: u8) -> Option<Self> {
        let tag = match value {
            0 => Self::Reset,

            1 => Self::Bold,
            2 => Self::Faint,
            3 => Self::Italic,
            4 => Self::Underline,
            5 => Self::SlowBlink,
            6 => Self::RapidBlink,
            7 => Self::Reverse,
            8 => Self::Conceal,
            9 => Self::CrossedOut,

            22 => Self::NotBold,
            23 => Self::NotItalic,
            24 => Self::NotUnderline,
            25 => Self::NotBlink,
            27 => Self::NotReverse,
            28 => Self::Reveal,
            29 => Self::NotCrossedOut,

            30 => Self::FgBlack,
            31 => Self::FgRed,
            32 => Self::FgGreen,
            33 => Self::FgYellow,
            34 => Self::FgBlue,
            35 => Self::FgMagenta,
            36 => Self::FgCyan,
            37 => Self::FgWhite,

            39 => Self::FgDefault,

            40 => Self::BgBlack,
            41 => Self::BgRed,
            42 => Self::BgGreen,
            43 => Self::BgYellow,
            44 => Self::BgBlue,
            45 => Self::BgMagenta,
            46 => Self::BgCyan,
            47 => Self::BgWhite,

            49 => Self::BgDefault,

            // 51 => Self::Framed,
            // 52 => Self::Encircled,
            // 53 => Self::Overlined,
            // 54 => Self::NotFramedOrEncircled,
            // 55 => Self::NotOverlined,

            //
            90 => Self::FgBrightBlack,
            91 => Self::FgBrightRed,
            92 => Self::FgBrightGreen,
            93 => Self::FgBrightYellow,
            94 => Self::FgBrightBlue,
            95 => Self::FgBrightMagenta,
            96 => Self::FgBrightCyan,
            97 => Self::FgBrightWhite,

            100 => Self::BgBrightBlack,
            101 => Self::BgBrightRed,
            102 => Self::BgBrightGreen,
            103 => Self::BgBrightYellow,
            104 => Self::BgBrightBlue,
            105 => Self::BgBrightMagenta,
            106 => Self::BgBrightCyan,
            107 => Self::BgBrightWhite,

            _ => return None,
        };

        Some(tag)
    }
}

impl std::fmt::Display for AnsiTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Reset => f.write_char('0'),

            Self::Bold => f.write_char('1'),
            Self::Faint => f.write_char('2'),
            Self::Italic => f.write_char('3'),
            Self::Underline => f.write_char('4'),
            Self::SlowBlink => f.write_char('5'),
            Self::RapidBlink => f.write_char('6'),
            Self::Reverse => f.write_char('7'),
            Self::Conceal => f.write_char('8'),
            Self::CrossedOut => f.write_char('9'),
            // Self::Framed => f.write_str("51"),
            // Self::Encircled => f.write_str("52"),
            // Self::Overlined => f.write_str("53"),

            //
            Self::NotBold => f.write_str("22"),
            Self::NotItalic => f.write_str("23"),
            Self::NotUnderline => f.write_str("24"),
            Self::NotBlink => f.write_str("25"),
            Self::NotReverse => f.write_str("27"),
            Self::Reveal => f.write_str("28"),
            Self::NotCrossedOut => f.write_str("29"),
            // Self::NotFramedOrEncircled => f.write_str("54"),
            // Self::NotOverlined => f.write_str("55"),

            //
            Self::FgBlack => f.write_str("30"),
            Self::FgRed => f.write_str("31"),
            Self::FgGreen => f.write_str("32"),
            Self::FgYellow => f.write_str("33"),
            Self::FgBlue => f.write_str("34"),
            Self::FgMagenta => f.write_str("35"),
            Self::FgCyan => f.write_str("36"),
            Self::FgWhite => f.write_str("37"),

            Self::FgBrightBlack => f.write_str("90"),
            Self::FgBrightRed => f.write_str("91"),
            Self::FgBrightGreen => f.write_str("92"),
            Self::FgBrightYellow => f.write_str("93"),
            Self::FgBrightBlue => f.write_str("94"),
            Self::FgBrightMagenta => f.write_str("95"),
            Self::FgBrightCyan => f.write_str("96"),
            Self::FgBrightWhite => f.write_str("97"),

            Self::FgDefault => f.write_str("39"),

            Self::BgBlack => f.write_str("40"),
            Self::BgRed => f.write_str("41"),
            Self::BgGreen => f.write_str("42"),
            Self::BgYellow => f.write_str("43"),
            Self::BgBlue => f.write_str("44"),
            Self::BgMagenta => f.write_str("45"),
            Self::BgCyan => f.write_str("46"),
            Self::BgWhite => f.write_str("47"),

            Self::BgBrightBlack => f.write_str("100"),
            Self::BgBrightRed => f.write_str("101"),
            Self::BgBrightGreen => f.write_str("102"),
            Self::BgBrightYellow => f.write_str("103"),
            Self::BgBrightBlue => f.write_str("104"),
            Self::BgBrightMagenta => f.write_str("105"),
            Self::BgBrightCyan => f.write_str("106"),
            Self::BgBrightWhite => f.write_str("107"),

            Self::BgDefault => f.write_str("49"),

            Self::Fg256(n) => f.write_fmt(format_args!("38;5;{}", n)),
            Self::Bg256(n) => f.write_fmt(format_args!("48;5;{}", n)),
            Self::FgTrueColor(r, g, b) => f.write_fmt(format_args!("38;2;{};{};{}", r, g, b)),
            Self::BgTrueColor(r, g, b) => f.write_fmt(format_args!("48;2;{};{};{}", r, g, b)),
        }
    }
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

    pub const fn len(&self) -> usize {
        self.inner.len()
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

    pub fn slice(&self, range: Range<usize>) -> &str {
        &self.inner[range]
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
        self.inner.extend([ANSI_START, ANSI_START2]);
        let _ = self.inner.write_fmt(format_args!("{tag}"));
        self.inner.push(ANSI_END);
    }

    pub fn insert_char(&mut self, i: usize, ch: char) {
        self.inner.insert(i, ch);
    }

    pub fn insert_str(&mut self, i: usize, s: &str) {
        self.inner.insert_str(i, s);
    }

    pub fn insert_tag(&mut self, i: usize, tag: AnsiTag) {
        self.inner
            .insert_str(i, &format!("{ANSI_START}{ANSI_START2}{tag}{ANSI_END}"));
    }

    pub fn extend<'a>(&mut self, iter: impl IntoIterator<Item = &'a str>) {
        self.inner.extend(iter);
    }

    pub fn textwrap(&mut self, width: u16) {
        textwrap::fill_inplace(&mut self.inner, width as usize);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

#[derive(Debug)]
pub enum AnsiEvent<'a> {
    Text(&'a str),
    Tag(AnsiTag),
}

pub struct AnsiParser<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    start: usize,
    tag: Option<AnsiTag>,
}

impl<'a> AnsiParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices(),
            start: 0,
            tag: None,
        }
    }

    pub const fn with_style(self) -> AnsiParserWithStyle<'a> {
        AnsiParserWithStyle::from_parser(self)
    }

    fn parse_ansi_code(&mut self) -> Option<(usize, AnsiTag)> {
        let Some((i, ANSI_START2)) = self.chars.next() else {
            return None;
        };

        let code_start = i + ANSI_START2.len_utf8();
        let Some(code_end) = self.find_ansi_end() else {
            return None;
        };

        let code = &self.input[code_start..code_end];
        let tag = if code.contains(";") {
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
                        (Some("38"), Some("5"), Some(Ok(index))) => Some(AnsiTag::Fg256(index)),
                        (Some("48"), Some("5"), Some(Ok(index))) => Some(AnsiTag::Bg256(index)),
                        _ => return None,
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
                            Some(AnsiTag::FgTrueColor(r, g, b))
                        }
                        (Some("48"), Some("2"), Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) => {
                            Some(AnsiTag::BgTrueColor(r, g, b))
                        }
                        _ => return None,
                    }
                }
                _ => return None,
            }
        } else {
            // Single code
            let Ok(num) = code.parse::<u8>() else {
                return None;
            };
            AnsiTag::from_u8(num)
        };

        tag.map(|tag| (code_end + ANSI_END.len_utf8(), tag))
    }

    fn find_ansi_end(&mut self) -> Option<usize> {
        let mut end = None;
        let max_code_len = 17; // Should be enough for code len
        for _ in 0..max_code_len {
            if let Some((i, ANSI_END)) = self.chars.next() {
                end = Some(i);
                break;
            }
        }
        end
    }
}

impl<'a> Iterator for AnsiParser<'a> {
    type Item = AnsiEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tag) = self.tag.take() {
            return Some(AnsiEvent::Tag(tag));
        }

        while let Some((i, c)) = self.chars.next() {
            if c == ANSI_START
                && let Some((end, tag)) = self.parse_ansi_code()
            {
                let event = if self.start < i {
                    // Set tag and return text
                    self.tag = Some(tag);
                    AnsiEvent::Text(&self.input[self.start..i])
                } else {
                    // Return tag
                    AnsiEvent::Tag(tag)
                };
                self.start = end;
                return Some(event);
            }
        }

        let remaining = &self.input[self.start..];
        self.start = self.input.len();

        if remaining.is_empty() {
            None
        } else {
            Some(AnsiEvent::Text(remaining))
        }
    }
}

pub struct AnsiParserWithStyle<'a> {
    parser: AnsiParser<'a>,
    style: Style,
}

impl<'a> AnsiParserWithStyle<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            parser: AnsiParser::new(input),
            style: Style::new(),
        }
    }

    pub const fn from_parser(parser: AnsiParser<'a>) -> Self {
        Self {
            parser,
            style: Style::new(),
        }
    }

    pub fn continue_with(&mut self, input: &'a str) -> &mut Self {
        self.parser = AnsiParser::new(input);
        self
    }
}

impl<'a> Iterator for AnsiParserWithStyle<'a> {
    type Item = (&'a str, Style);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.parser.next() {
            match event {
                AnsiEvent::Text(s) => {
                    return Some((s, self.style));
                }
                AnsiEvent::Tag(tag) => match tag {
                    AnsiTag::Reset => {
                        self.style = Style::new();
                    }
                    AnsiTag::Bold => {
                        self.style.add_modifier.insert(Modifier::BOLD);
                    }
                    AnsiTag::Faint => {
                        self.style.add_modifier.insert(Modifier::DIM);
                    }
                    AnsiTag::Italic => {
                        self.style.add_modifier.insert(Modifier::ITALIC);
                    }
                    AnsiTag::Underline => {
                        self.style.add_modifier.insert(Modifier::UNDERLINED);
                    }
                    AnsiTag::SlowBlink => {
                        self.style.add_modifier.insert(Modifier::SLOW_BLINK);
                    }
                    AnsiTag::RapidBlink => {
                        self.style.add_modifier.insert(Modifier::RAPID_BLINK);
                    }
                    AnsiTag::Reverse => {
                        self.style.add_modifier.insert(Modifier::REVERSED);
                    }
                    AnsiTag::Conceal => {
                        self.style.add_modifier.insert(Modifier::HIDDEN);
                    }
                    AnsiTag::CrossedOut => {
                        self.style.add_modifier.insert(Modifier::CROSSED_OUT);
                    }
                    // AnsiTag::Framed => todo!(),
                    // AnsiTag::Encircled => todo!(),
                    // AnsiTag::Overlined => todo!(),
                    AnsiTag::NotBold => {
                        self.style.add_modifier.remove(Modifier::BOLD);
                    }
                    AnsiTag::NotItalic => {
                        self.style.add_modifier.remove(Modifier::ITALIC);
                    }
                    AnsiTag::NotUnderline => {
                        self.style.add_modifier.remove(Modifier::UNDERLINED);
                    }
                    AnsiTag::NotBlink => {
                        self.style.add_modifier.remove(Modifier::SLOW_BLINK);
                        self.style.add_modifier.remove(Modifier::RAPID_BLINK);
                    }
                    AnsiTag::NotReverse => {
                        self.style.add_modifier.remove(Modifier::REVERSED);
                    }
                    AnsiTag::Reveal => {
                        self.style.add_modifier.remove(Modifier::HIDDEN);
                    }
                    AnsiTag::NotCrossedOut => {
                        self.style.add_modifier.remove(Modifier::CROSSED_OUT);
                    }
                    // AnsiTag::NotFramedOrEncircled => todo!(),
                    // AnsiTag::NotOverlined => todo!(),
                    AnsiTag::FgBlack => {
                        self.style.fg = Some(Color::Black);
                    }
                    AnsiTag::FgRed => {
                        self.style.fg = Some(Color::Red);
                    }
                    AnsiTag::FgGreen => {
                        self.style.fg = Some(Color::Green);
                    }
                    AnsiTag::FgYellow => {
                        self.style.fg = Some(Color::Yellow);
                    }
                    AnsiTag::FgBlue => {
                        self.style.fg = Some(Color::Blue);
                    }
                    AnsiTag::FgMagenta => {
                        self.style.fg = Some(Color::Magenta);
                    }
                    AnsiTag::FgCyan => {
                        self.style.fg = Some(Color::Cyan);
                    }
                    AnsiTag::FgWhite => {
                        self.style.fg = Some(Color::Gray);
                    }
                    AnsiTag::FgBrightBlack => {
                        self.style.fg = Some(Color::DarkGray);
                    }
                    AnsiTag::FgBrightRed => {
                        self.style.fg = Some(Color::LightRed);
                    }
                    AnsiTag::FgBrightGreen => {
                        self.style.fg = Some(Color::LightGreen);
                    }
                    AnsiTag::FgBrightYellow => {
                        self.style.fg = Some(Color::LightYellow);
                    }
                    AnsiTag::FgBrightBlue => {
                        self.style.fg = Some(Color::LightBlue);
                    }
                    AnsiTag::FgBrightMagenta => {
                        self.style.fg = Some(Color::LightMagenta);
                    }
                    AnsiTag::FgBrightCyan => {
                        self.style.fg = Some(Color::LightCyan);
                    }
                    AnsiTag::FgBrightWhite => {
                        self.style.fg = Some(Color::White);
                    }
                    AnsiTag::FgDefault => {
                        self.style.fg = Some(Color::Reset);
                    }
                    AnsiTag::BgBlack => {
                        self.style.bg = Some(Color::Black);
                    }
                    AnsiTag::BgRed => {
                        self.style.bg = Some(Color::Red);
                    }
                    AnsiTag::BgGreen => {
                        self.style.bg = Some(Color::Green);
                    }
                    AnsiTag::BgYellow => {
                        self.style.bg = Some(Color::Yellow);
                    }
                    AnsiTag::BgBlue => {
                        self.style.bg = Some(Color::Blue);
                    }
                    AnsiTag::BgMagenta => {
                        self.style.bg = Some(Color::Magenta);
                    }
                    AnsiTag::BgCyan => {
                        self.style.bg = Some(Color::Cyan);
                    }
                    AnsiTag::BgWhite => {
                        self.style.bg = Some(Color::Gray);
                    }
                    AnsiTag::BgBrightBlack => {
                        self.style.bg = Some(Color::DarkGray);
                    }
                    AnsiTag::BgBrightRed => {
                        self.style.bg = Some(Color::LightRed);
                    }
                    AnsiTag::BgBrightGreen => {
                        self.style.bg = Some(Color::LightGreen);
                    }
                    AnsiTag::BgBrightYellow => {
                        self.style.bg = Some(Color::LightYellow);
                    }
                    AnsiTag::BgBrightBlue => {
                        self.style.bg = Some(Color::LightBlue);
                    }
                    AnsiTag::BgBrightMagenta => {
                        self.style.bg = Some(Color::LightMagenta);
                    }
                    AnsiTag::BgBrightCyan => {
                        self.style.bg = Some(Color::LightCyan);
                    }
                    AnsiTag::BgBrightWhite => {
                        self.style.bg = Some(Color::White);
                    }
                    AnsiTag::BgDefault => {
                        self.style.bg = Some(Color::Reset);
                    }
                    AnsiTag::Fg256(i) => {
                        self.style.fg = Some(Color::Indexed(i));
                    }
                    AnsiTag::Bg256(i) => {
                        self.style.bg = Some(Color::Indexed(i));
                    }
                    AnsiTag::FgTrueColor(r, g, b) => {
                        self.style.fg = Some(Color::Rgb(r, g, b));
                    }
                    AnsiTag::BgTrueColor(r, g, b) => {
                        self.style.bg = Some(Color::Rgb(r, g, b));
                    }
                },
            }
        }

        None
    }
}

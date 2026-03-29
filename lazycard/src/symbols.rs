#![allow(unused)]

pub const TAB: &str = "Tab";
pub const ALT: &str = "⎇";
pub const CTRL: &str = "^";
pub const SHIFT: &str = "⇧";
pub const ENTER: &str = "↵";
pub const SPACE: &str = "Space";
pub const ESCAPE: &str = "Esc";
pub const DELETE: &str = "Del";
pub const ARROW_UP: &str = "￪";
pub const ARROW_DOWN: &str = "￬";
pub const ARROW_RIGHT: &str = "➔";
pub const ARROW_DOWN_UP: &str = "⇵";
pub const ARROW_RIGHT_LEFT: &str = "⇄";
pub const ARROW_LEFT_RIGHT: &str = "⇆";
pub const ARROW_HEAD_UP: &str = "⌃";
pub const ARROW_HEAD_DOWN: &str = "⌄";

pub const SELECTED: &str = ">";
pub const SELECTION: &str = "|";
pub const STAR: &str = "★";
pub const STAR_BIG: &str = "🟊";
pub const CHECKMARK_YES: &str = "🗸";
pub const CHECKMARK_NO: &str = "✗";

pub const fn checkmark(v: bool) -> &'static str {
    if v { CHECKMARK_YES } else { CHECKMARK_NO }
}

macro_rules! alt {
    ($s:expr) => {{
        const _: &str = $s;
        constcat::concat!(crate::symbols::ALT, $s)
    }};
}

macro_rules! ctrl {
    ($s:expr) => {{
        const _: &str = $s;
        constcat::concat!(crate::symbols::CTRL, $s)
    }};
}

macro_rules! shift {
    ($s:expr) => {{
        const _: &str = $s;
        constcat::concat!("(", crate::symbols::SHIFT, ")", $s)
    }};
}

macro_rules! repeat {
    ($s:expr, 0) => {{
        const _: &str = $s;
        ""
    }};
    ($s:expr, 1) => {{
        const _: &str = $s;
        $s
    }};
    ($s:expr, 2) => {{
        const _: &str = $s;
        constcat::concat!($s, $s)
    }};
    ($s:expr, 3) => {{
        const _: &str = $s;
        constcat::concat!($s, $s, $s)
    }};
    ($s:expr, 4) => {{
        const _: &str = $s;
        constcat::concat!($s, $s, $s, $s)
    }};
    ($s:expr, 5) => {{
        const _: &str = $s;
        constcat::concat!($s, $s, $s, $s, $s)
    }};
}

pub(crate) use alt;
pub(crate) use constcat::concat;
pub(crate) use ctrl;
pub(crate) use repeat;
pub(crate) use shift;

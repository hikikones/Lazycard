/// Formats an integer to its string representation.
pub fn format_int(i: impl itoa::Integer, mut f: impl FnMut(&str)) {
    let mut buffer = itoa::Buffer::new();
    f(buffer.format(i))
}

/// Formats two integers to its string representations.
pub fn format_int2(i1: impl itoa::Integer, i2: impl itoa::Integer, mut f: impl FnMut(&str, &str)) {
    let (mut b1, mut b2) = (itoa::Buffer::new(), itoa::Buffer::new());
    f(b1.format(i1), b2.format(i2))
}

pub struct Formatter(String);

impl Formatter {
    pub const fn new() -> Self {
        Self(String::new())
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn slice(&self, range: std::ops::Range<usize>) -> &str {
        &self.0[range]
    }

    pub fn push_str(&mut self, s: &str) -> std::ops::Range<usize> {
        let start = self.0.len();
        self.0.push_str(s);
        start..self.0.len()
    }

    pub fn push_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::ops::Range<usize> {
        use std::fmt::Write;

        let start = self.0.len();
        let _ = self.0.write_fmt(args);
        start..self.0.len()
    }

    pub fn push_fmt2(
        &mut self,
        args1: std::fmt::Arguments<'_>,
        args2: std::fmt::Arguments<'_>,
    ) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        use std::fmt::Write;

        let start = self.0.len();
        let _ = self.0.write_fmt(args1);
        let middle = self.0.len();
        let _ = self.0.write_fmt(args2);
        (start..middle, middle..self.0.len())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

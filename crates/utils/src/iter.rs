use std::{iter::Peekable, str::CharIndices};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

#[derive(Debug)]
pub struct CustomGraphemeIter<'a> {
    graphemes: Peekable<GraphemeIndices<'a>>,
    current: Option<(usize, &'a str)>,
    previous: Option<&'a str>,
}

impl<'a> CustomGraphemeIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            graphemes: input.grapheme_indices(true).peekable(),
            current: None,
            previous: None,
        }
    }

    pub const fn previous(&self) -> Option<&'a str> {
        self.previous
    }

    pub fn peek(&mut self) -> Option<&'a str> {
        self.graphemes.peek().map(|(_, g)| *g)
    }

    pub fn next_if(&mut self, func: impl Fn(&str) -> bool) -> Option<(usize, &'a str)> {
        if let Some((_, peek)) = self.graphemes.peek() {
            if func(peek) {
                return self.next();
            }
        }
        None
    }

    pub fn next_if_eq(&mut self, g: &str) -> Option<(usize, &'a str)> {
        self.next_if(|n| n == g)
    }

    pub fn next_if_newline(&mut self) -> Option<(usize, &'a str)> {
        self.next_if(|n| n.contains('\n'))
    }

    pub fn find(&mut self, g: &str) -> Option<(usize, &'a str)> {
        self.find_by(|n| n == g)
    }

    pub fn find_by(&mut self, func: impl Fn(&str) -> bool) -> Option<(usize, &'a str)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if func(n) {
                return Some((i, n));
            }
        }
    }

    pub fn find_newline(&mut self) -> Option<(usize, &'a str)> {
        self.find_by(|n| n.contains('\n'))
    }

    pub fn find_with_previous(
        &mut self,
        g: &str,
        prev_func: impl Fn(&str) -> bool,
    ) -> Option<(usize, &'a str)> {
        self.find_by_with_previous(|n| n == g, prev_func)
    }

    pub fn find_by_with_previous(
        &mut self,
        next_func: impl Fn(&str) -> bool,
        prev_func: impl Fn(&str) -> bool,
    ) -> Option<(usize, &'a str)> {
        loop {
            let Some((i, n)) = self.find_by(&next_func) else {
                return None;
            };

            if let Some(p) = self.previous {
                if prev_func(p) {
                    return Some((i, n));
                }
            };
        }
    }

    pub fn find_pattern(&mut self, prev: &str, next: &str) -> Option<(usize, &'a str, &'a str)> {
        self.find_pattern_by(|p, n| p == prev && n == next)
    }

    pub fn find_pattern_by(
        &mut self,
        func: impl Fn(&str, &str) -> bool,
    ) -> Option<(usize, &'a str, &'a str)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if let Some(p) = self.previous {
                if func(p, n) {
                    return Some((i, p, n));
                }
            }
        }
    }

    pub fn find_consecutive(&mut self, g: &str, n: usize) -> Option<(usize, &'a str)> {
        self.find_consecutive_by(|s| s == g, n)
    }

    pub fn find_consecutive_by(
        &mut self,
        func: impl Fn(&str) -> bool,
        n: usize,
    ) -> Option<(usize, &'a str)> {
        match n {
            0 => None,
            1 => self.find_by(func),
            _ => loop {
                if self.find_by(&func).is_none() {
                    return None;
                };

                let mut count = 1;
                while let Some((i, g)) = self.next_if(&func) {
                    count += 1;
                    if count == n {
                        return Some((i, g));
                    }
                }
            },
        }
    }

    pub fn count_consecutive(&mut self, g: &str, max: usize) -> usize {
        self.count_consecutive_by(|n| n == g, max)
    }

    pub fn count_consecutive_by(&mut self, g: impl Fn(&str) -> bool, max: usize) -> usize {
        if max == 0 {
            return 0;
        }

        let mut count = 0;
        while self.next_if(&g).is_some() {
            count += 1;
            if count == max {
                break;
            }
        }
        count
    }
}

impl<'a> Iterator for CustomGraphemeIter<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.graphemes.next();

        if next.is_none() {
            return None;
        }

        self.previous = self.current.map(|(_, g)| g);
        self.current = next;
        next
    }
}

#[derive(Debug)]
pub struct CustomCharsIter<'a> {
    chars: Peekable<CharIndices<'a>>,
    current: Option<(usize, char)>,
    previous: Option<char>,
}

impl<'a> CustomCharsIter<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            current: None,
            previous: None,
        }
    }

    pub const fn previous(&self) -> Option<char> {
        self.previous
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied().map(|(_, c)| c)
    }

    pub fn next_if(&mut self, f: impl Fn(char) -> bool) -> Option<(usize, char)> {
        if let Some(p) = self.peek() {
            if f(p) {
                return self.next();
            }
        }
        None
    }

    pub fn next_if_eq(&mut self, c: char) -> Option<(usize, char)> {
        self.next_if(|n| n == c)
    }

    pub fn find(&mut self, c: char) -> Option<(usize, char)> {
        self.find_by(|n| n == c)
    }

    pub fn find_by(&mut self, f: impl Fn(char) -> bool) -> Option<(usize, char)> {
        loop {
            let Some((i, n)) = self.next() else {
                return None;
            };

            if f(n) {
                return Some((i, n));
            }
        }
    }

    pub fn find_with_previous(
        &mut self,
        c: char,
        prev_func: impl Fn(char) -> bool,
    ) -> Option<(usize, char)> {
        self.find_by_with_previous(|n| n == c, prev_func)
    }

    pub fn find_by_with_previous(
        &mut self,
        next_func: impl Fn(char) -> bool,
        prev_func: impl Fn(char) -> bool,
    ) -> Option<(usize, char)> {
        loop {
            let Some((i, n)) = self.find_by(&next_func) else {
                return None;
            };

            if let Some(p) = self.previous {
                if prev_func(p) {
                    return Some((i, n));
                }
            };
        }
    }

    pub fn find_consecutive(&mut self, c: char, n: usize) -> Option<(usize, char)> {
        self.find_consecutive_by(|s| s == c, n)
    }

    pub fn find_consecutive_by(
        &mut self,
        f: impl Fn(char) -> bool,
        n: usize,
    ) -> Option<(usize, char)> {
        match n {
            0 => None,
            1 => self.find_by(f),
            _ => loop {
                if self.find_by(&f).is_none() {
                    return None;
                };

                let mut count = 1;
                while let Some((i, c)) = self.next_if(&f) {
                    count += 1;
                    if count == n {
                        return Some((i, c));
                    }
                }
            },
        }
    }

    pub fn count_consecutive(&mut self, c: char, max: usize) -> usize {
        self.count_consecutive_by(|n| n == c, max)
    }

    pub fn count_consecutive_by(&mut self, c: impl Fn(char) -> bool, max: usize) -> usize {
        if max == 0 {
            return 0;
        }

        let mut count = 0;
        while self.next_if(&c).is_some() {
            count += 1;
            if count == max {
                break;
            }
        }
        count
    }
}

impl<'a> Iterator for CustomCharsIter<'a> {
    type Item = (usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.chars.next();

        if next.is_none() {
            return None;
        }

        self.previous = self.current.map(|(_, c)| c);
        self.current = next;
        next
    }
}

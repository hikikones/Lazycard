use std::{iter::Peekable, str::CharIndices};

use ratatui::layout::Alignment;

#[derive(Debug)]
pub enum BlockElement<'a> {
    Paragraph { alignment: Alignment, text: &'a str },
    Code { language: &'a str, text: &'a str },
}

pub struct BlockParser<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> BlockParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
        }
    }
}

impl<'a> Iterator for BlockParser<'a> {
    type Item = BlockElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.chars.next() {
            match c {
                '\n' | ' ' | '\t' => {
                    continue;
                }
                '`' => {
                    let ticks = 1 + count_ticks(&mut self.chars);
                    if ticks < 3 {
                        return Some(parse_paragraph(i, c, self.input, &mut self.chars));
                    }
                    return Some(parse_code_block(i, ticks, self.input, &mut self.chars));
                }
                _ => {
                    return Some(parse_paragraph(i, c, self.input, &mut self.chars));
                }
            }
        }

        None
    }
}

fn parse_paragraph<'a>(
    i: usize,
    c: char,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> BlockElement<'a> {
    let (alignment, offset) = {
        if c == '>' {
            (Alignment::Right, 1)
        } else if c == '|' {
            (Alignment::Center, 1)
        } else {
            (Alignment::Left, 0)
        }
    };

    let paragraph_start = i + offset;
    loop {
        if chars.find(|&(_, c)| c == '\n').is_none() {
            return BlockElement::Paragraph {
                alignment,
                text: input[paragraph_start..].trim(),
            };
        };

        if let Some((ni, '\n')) = chars.next() {
            return BlockElement::Paragraph {
                alignment,
                text: input[paragraph_start..ni - 1].trim(),
            };
        }
    }
}

fn parse_code_block<'a>(
    i: usize,
    ticks: usize,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> BlockElement<'a> {
    let lang_start = i + ticks;
    let Some((i, _)) = chars.find(|(_, c)| *c == '\n') else {
        return BlockElement::Code {
            language: input[lang_start..].trim(),
            text: "",
        };
    };

    let language = input[lang_start..i].trim();
    let code_start = i + 1;
    loop {
        let Some((end, _)) = chars.find(|(_, c)| *c == '\n') else {
            return BlockElement::Code {
                language,
                text: &input[code_start..],
            };
        };

        let end_ticks = count_ticks(chars);
        if end_ticks == ticks {
            let Some((_, c)) = chars.next() else {
                return BlockElement::Code {
                    language,
                    text: &input[code_start..end],
                };
            };

            if c == '\n' {
                return BlockElement::Code {
                    language,
                    text: &input[code_start..end],
                };
            }
        }
    }
}

fn count_ticks(chars: &mut Peekable<CharIndices>) -> usize {
    let mut count = 0;
    while chars.next_if(|(_, c)| *c == '`').is_some() {
        count += 1;
    }
    count
}

#[derive(Debug)]
pub enum InlineElement<'a> {
    Text(&'a str),
    Bold(&'a str),
    Cursive(&'a str),
}

#[derive(Debug, Clone, Copy)]
enum InlineTag {
    Text,
    Bold,
    Cursive,
}

pub struct InlineParser<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    tag: InlineTag,
    start: usize,
}

impl<'a> InlineParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices(),
            tag: InlineTag::Text,
            start: 0,
        }
    }

    pub fn continue_with(&mut self, input: &'a str) {
        self.input = input;
        self.chars = input.char_indices();
        self.start = 0;
    }
}

impl<'a> Iterator for InlineParser<'a> {
    type Item = InlineElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((_, c)) = self.chars.next() else {
                let text = &self.input[self.start..];

                if text.is_empty() {
                    return None;
                }

                self.start = self.input.len();
                let element = match self.tag {
                    InlineTag::Text => InlineElement::Text(text),
                    InlineTag::Bold => InlineElement::Bold(text),
                    InlineTag::Cursive => InlineElement::Cursive(text),
                };
                return Some(element);
            };

            match self.tag {
                InlineTag::Text => match c {
                    '*' => {
                        if let Some((ni, '*')) = self.chars.next() {
                            self.tag = InlineTag::Bold;
                            let text = &self.input[self.start..ni - 1];
                            self.start = ni + 1;

                            if !text.is_empty() {
                                return Some(InlineElement::Text(text));
                            }
                        };
                    }
                    '_' => {
                        if let Some((ni, '_')) = self.chars.next() {
                            self.tag = InlineTag::Cursive;
                            let text = &self.input[self.start..ni - 1];
                            self.start = ni + 1;

                            if !text.is_empty() {
                                return Some(InlineElement::Text(text));
                            }
                        };
                    }
                    _ => {}
                },
                InlineTag::Bold => {
                    let bold_start = self.start;
                    loop {
                        if self.chars.find(|&(_, c)| c == '*').is_none() {
                            break;
                        };

                        if let Some((ni, '*')) = self.chars.next() {
                            self.tag = InlineTag::Text;
                            self.start = ni + 1;
                            return Some(InlineElement::Bold(&self.input[bold_start..ni - 1]));
                        }
                    }
                }
                InlineTag::Cursive => {
                    let cursive_start = self.start;
                    loop {
                        if self.chars.find(|&(_, c)| c == '_').is_none() {
                            break;
                        };

                        if let Some((ni, '_')) = self.chars.next() {
                            self.tag = InlineTag::Text;
                            self.start = ni + 1;
                            return Some(InlineElement::Cursive(
                                &self.input[cursive_start..ni - 1],
                            ));
                        }
                    }
                }
            }
        }
    }
}

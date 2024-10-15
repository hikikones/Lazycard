#[derive(Debug)]
pub enum BlockElement<'a> {
    Paragraph {
        spans: Vec<InlineElement<'a>>,
        alignment: Alignment,
    },
    Code {
        language: &'a str,
        text: &'a str,
    },
}

#[derive(Debug)]
pub enum InlineElement<'a> {
    Text(&'a str),
    Bold(&'a str),
    Cursive(&'a str),
}

#[derive(Debug)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

pub fn parse<'a>(input: &'a str) -> Vec<BlockElement<'a>> {
    let mut input = input.trim();
    let mut chars = input.char_indices();
    let mut blocks = Vec::new();

    while let Some((i, c)) = chars.next() {
        match c {
            '\n' | ' ' | '\t' => {
                continue;
            }
            '>' => {
                input = &input[i + 1..];
                blocks.push(parse_paragraph(&mut input, Alignment::Right));
                chars = input.char_indices();
            }
            '|' => {
                input = &input[i + 1..];
                blocks.push(parse_paragraph(&mut input, Alignment::Center));
                chars = input.char_indices();
            }
            '`' => {
                let Some((_, '`')) = chars.next() else {
                    input = &input[i..];
                    blocks.push(parse_paragraph(&mut input, Alignment::Left));
                    chars = input.char_indices();
                    continue;
                };

                let Some((_, '`')) = chars.next() else {
                    input = &input[i..];
                    blocks.push(parse_paragraph(&mut input, Alignment::Left));
                    chars = input.char_indices();
                    continue;
                };

                input = &input[i + 3..];
                blocks.push(parse_code_block(&mut input));
                chars = input.char_indices();
                continue;
            }
            _ => {
                input = &input[i..];
                blocks.push(parse_paragraph(&mut input, Alignment::Left));
                chars = input.char_indices();
            }
        }
    }

    blocks
}

fn parse_paragraph<'a>(input: &mut &'a str, alignment: Alignment) -> BlockElement<'a> {
    let Some(end) = input.find("\n\n") else {
        let text = input.trim();
        *input = "";
        return BlockElement::Paragraph {
            spans: parse_spans(text),
            alignment,
        };
    };

    let text = input[..end].trim();
    *input = &input[end + 2..];
    BlockElement::Paragraph {
        spans: parse_spans(text),
        alignment,
    }
}

fn parse_code_block<'a>(input: &mut &'a str) -> BlockElement<'a> {
    let mut chars = input.char_indices();
    let mut start_ticks = 3;
    let mut i = 0;
    let mut c = '`';

    loop {
        let Some((ni, nc)) = chars.next() else {
            return BlockElement::Code {
                language: "",
                text: "",
            };
        };

        if nc == '`' {
            start_ticks += 1;
            continue;
        }

        i = ni;
        c = nc;

        break;
    }

    let language = {
        if c == '\n' {
            i += 1;
            ""
        } else {
            match chars.find(|&(_, c)| c == '\n') {
                Some((ni, _)) => {
                    let lang = &input[i..ni];
                    i = ni + 1;
                    lang.trim()
                }
                None => {
                    let lang = &input[i..];
                    *input = "";
                    return BlockElement::Code {
                        language: lang.trim(),
                        text: "",
                    };
                }
            }
        }
    };

    loop {
        let Some((mut end, _)) = chars.find(|&(_, c)| c == '\n') else {
            let text = &input[i..];
            *input = "";
            return BlockElement::Code { language, text };
        };

        let mut end_ticks = 0;
        loop {
            let Some((ni, nc)) = chars.next() else {
                let text = &input[i..];
                *input = "";
                return BlockElement::Code { language, text };
            };

            if nc == '`' {
                end_ticks += 1;
                if end_ticks == start_ticks {
                    let text = &input[i..end];
                    *input = &input[end + 1 + end_ticks..];
                    return BlockElement::Code { language, text };
                }
                continue;
            } else if nc == '\n' {
                end = ni;
                end_ticks = 0;
                continue;
            }

            break;
        }
    }
}

fn parse_spans<'a>(input: &'a str) -> Vec<InlineElement<'a>> {
    let mut chars = input.char_indices();
    let mut spans = Vec::new();

    let mut text_start = 0;
    loop {
        let Some((i, c)) = chars.next() else {
            let text = &input[text_start..];
            if !text.is_empty() {
                spans.push(InlineElement::Text(text));
            }
            break;
        };

        match c {
            '_' => {
                if let Some((ni, '_')) = chars.next() {
                    let text = &input[text_start..ni - 1];
                    if !text.is_empty() {
                        spans.push(InlineElement::Text(text));
                    }

                    let cursive_start = ni + 1;
                    loop {
                        if chars.find(|&(_, c)| c == '_').is_none() {
                            spans.push(InlineElement::Cursive(&input[cursive_start..]));
                            text_start = input.len();
                            break;
                        };

                        if let Some((ni, '_')) = chars.next() {
                            text_start = ni + 1;
                            spans.push(InlineElement::Cursive(&input[cursive_start..ni - 1]));
                            break;
                        }
                    }
                };
            }
            '*' => {
                if let Some((ni, '*')) = chars.next() {
                    let text = &input[text_start..ni - 1];
                    if !text.is_empty() {
                        spans.push(InlineElement::Text(text));
                    }

                    let cursive_start = ni + 1;
                    loop {
                        if chars.find(|&(_, c)| c == '*').is_none() {
                            spans.push(InlineElement::Bold(&input[cursive_start..]));
                            text_start = input.len();
                            break;
                        };

                        if let Some((ni, '*')) = chars.next() {
                            text_start = ni + 1;
                            spans.push(InlineElement::Bold(&input[cursive_start..ni - 1]));
                            break;
                        }
                    }
                };
            }
            _ => {}
        }
    }

    spans
}

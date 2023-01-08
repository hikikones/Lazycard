use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

pub struct Markdown {
    theme_set: ThemeSet,
    syntax_set: SyntaxSet,
}

impl Markdown {
    pub fn to_html(&self, text: &str) -> String {
        let custom_parser = CustomParser {
            parser: Parser::new_ext(text, Options::all()),
            syntax_set: &self.syntax_set,
            theme: &self.theme_set.themes["base16-eighties.dark"],
        };

        let mut html_output = String::with_capacity(text.len() * 3 / 2);
        html::push_html(&mut html_output, custom_parser);

        html_output
    }
}

impl Default for Markdown {
    fn default() -> Self {
        Self {
            theme_set: ThemeSet::load_defaults(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }
}

struct CustomParser<'e> {
    parser: Parser<'e, 'e>,
    syntax_set: &'e SyntaxSet,
    theme: &'e Theme,
}

impl<'e> Iterator for CustomParser<'e> {
    type Item = Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next) = self.parser.next() else {
            return None;
        };

        match &next {
            Event::Start(tag) => match tag {
                Tag::CodeBlock(_) => {
                    return self.parse_code_block();
                }
                Tag::Image(link, src, title) => {
                    if !is_url::is_url(src) {
                        return Some(Event::Start(Tag::Image(
                            link.clone(),
                            CowStr::Boxed(format!("{}/{}", config::ASSETS_DIR, src).into()),
                            title.clone(),
                        )));
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Some(next)
    }
}

impl<'e> CustomParser<'e> {
    fn parse_code_block<'a>(&'a mut self) -> Option<Event<'e>> {
        let mut to_highlight = String::new();

        while let Some(ref event) = self.parser.next() {
            match event {
                Event::Text(t) => {
                    to_highlight.push_str(t);
                }
                Event::End(Tag::CodeBlock(token)) => {
                    let syntax = if let CodeBlockKind::Fenced(val) = token {
                        self.syntax_set
                            .find_syntax_by_extension(val)
                            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
                    } else {
                        self.syntax_set.find_syntax_plain_text()
                    };

                    let html = highlighted_html_for_string(
                        &to_highlight,
                        self.syntax_set,
                        &syntax,
                        self.theme,
                    )
                    .unwrap();

                    return Some(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                }
                _ => {}
            }
        }

        None
    }
}

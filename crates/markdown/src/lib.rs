use once_cell::sync::Lazy;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| ThemeSet::load_defaults());

pub fn to_html(markdown: &str) -> String {
    let mut html_output = String::with_capacity(markdown.len() * 3 / 2);
    html::push_html(
        &mut html_output,
        CustomParser(Parser::new_ext(markdown, Options::all())),
    );
    html_output
}

struct CustomParser<'e>(Parser<'e, 'e>);

impl<'e> Iterator for CustomParser<'e> {
    type Item = Event<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next) = self.0.next() else {
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

        while let Some(ref event) = self.0.next() {
            match event {
                Event::Text(t) => {
                    to_highlight.push_str(t);
                }
                Event::End(Tag::CodeBlock(token)) => {
                    let ss = &SYNTAX_SET;
                    let syntax = if let CodeBlockKind::Fenced(val) = token {
                        ss.find_syntax_by_extension(val)
                            .unwrap_or_else(|| ss.find_syntax_plain_text())
                    } else {
                        ss.find_syntax_plain_text()
                    };

                    let theme = &THEME_SET.themes["base16-eighties.dark"];
                    let html =
                        highlighted_html_for_string(&to_highlight, ss, &syntax, theme).unwrap();

                    return Some(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                }
                _ => {}
            }
        }

        None
    }
}

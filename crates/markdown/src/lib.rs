pub fn to_html(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);

    let mut html_buf = String::new();
    pulldown_cmark::html::push_html(&mut html_buf, parser);

    html_buf
}

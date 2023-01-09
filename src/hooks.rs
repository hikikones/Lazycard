use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::ScopeState;

use config::Config;
use database::Database;
use markdown::Markdown;

pub fn use_config(cx: &ScopeState) -> &Rc<RefCell<Config>> {
    cx.use_hook(|| cx.consume_context::<Rc<RefCell<Config>>>().unwrap())
}

pub fn use_database(cx: &ScopeState) -> &Rc<RefCell<Database>> {
    cx.use_hook(|| cx.consume_context::<Rc<RefCell<Database>>>().unwrap())
}

pub fn use_markdown(cx: &ScopeState) -> &Rc<Markdown> {
    cx.use_hook(|| cx.consume_context::<Rc<Markdown>>().unwrap())
}

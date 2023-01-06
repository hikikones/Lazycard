use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::ScopeState;

use config::Config;
use database::Database;

pub fn use_config(cx: &ScopeState) -> Rc<RefCell<Config>> {
    cx.use_hook(|_| cx.consume_context::<Rc<RefCell<Config>>>().unwrap())
        .clone()
}

pub fn use_database(cx: &ScopeState) -> Rc<Database> {
    cx.use_hook(|_| cx.consume_context::<Rc<Database>>().unwrap())
        .clone()
}

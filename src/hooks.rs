use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::ScopeState;

use config::Config;
use database::Database;

pub fn use_config(cx: &ScopeState) -> &Rc<Config> {
    cx.use_hook(|| cx.consume_context::<Rc<Config>>().unwrap())
}

pub fn use_database(cx: &ScopeState) -> &Rc<RefCell<Database>> {
    cx.use_hook(|| cx.consume_context::<Rc<RefCell<Database>>>().unwrap())
}

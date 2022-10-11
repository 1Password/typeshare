#![recursion_limit = "1024"]
#![cfg(target_arch = "wasm32")]

use crate::app::TypeshareReplApp;
use console_error_panic_hook::set_once as set_panic_hook;
use std::rc::Rc;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::{highlighting::ThemeSet, parsing::SyntaxDefinition};
use typeshare_core::language::Go;
use typeshare_core::{
    language::{Kotlin, Swift, TypeScript},
    process_input,
};
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::prelude::*;
use yewdux::prelude::*;

mod app;

fn main() {
    set_panic_hook();
    let web_options = eframe::WebOptions::default();
    let mut go = Go::default();
    go.package = "repl".to_string();
    let mut kotlin = Kotlin::default();
    kotlin.package = "com.agilebits.repl".to_string();
    let swift = Swift::default();
    let typescript = TypeScript::default();
    let _ = eframe::start_web(
        "typeshare_repl",
        web_options,
        Box::new(|_| Box::new(TypeshareReplApp::new(typescript, swift, kotlin, go))),
    );
}

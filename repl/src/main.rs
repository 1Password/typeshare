#![recursion_limit = "1024"]
#![cfg(target_arch = "wasm32")]

use crate::app::TypeshareReplApp;
use console_error_panic_hook::set_once as set_panic_hook;
use typeshare_core::{
    language::{Go, Kotlin, Swift, TypeScript},
};


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

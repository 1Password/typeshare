use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use yew::{
    prelude::*,
    web_sys::{self, Document, Window},
};
use yewdux::prelude::*;

mod toggle;

pub use toggle::Toggle;

#[derive(Clone, Properties)]
pub struct PageThemeHandler {
    pub page_theme: Rc<PageTheme>,
    pub page_theme_dispatch: Dispatch<PersistentStore<PageTheme>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PageTheme {
    Dark,
    Light,
}

impl Default for PageTheme {
    fn default() -> Self {
        Self::Dark
    }
}

impl Persistent for PageTheme {}

impl PageTheme {
    pub fn toggle(&mut self) {
        *self = match *self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        };
    }
}

impl AsRef<str> for PageTheme {
    fn as_ref(&self) -> &str {
        match self {
            PageTheme::Dark => "dark",
            PageTheme::Light => "light",
        }
    }
}

impl Display for PageTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

pub fn set_page_theme(theme: PageTheme) {
    let document_element = web_sys::window()
        .as_ref()
        .map(Window::document)
        .flatten()
        .as_ref()
        .map(Document::document_element)
        .flatten()
        .map(|e| e.set_attribute("data-theme", theme.as_ref()));

    if document_element.is_none() {
        web_sys::console::log_1(&JsValue::from("Could not find document for setting theme"));
    }
}

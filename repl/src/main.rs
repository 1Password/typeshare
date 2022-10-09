#![recursion_limit = "1024"]

use crate::raw_html::RawHtml;
use crate::theme::{PageTheme, PageThemeHandler};
use console_error_panic_hook::set_once as set_panic_hook;
use std::rc::Rc;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use syntect::{highlighting::ThemeSet, parsing::SyntaxDefinition};
use typeshare_core::{
    language::{Kotlin, Swift, TypeScript},
    process_input,
};
use ybc::TileCtx::{Ancestor, Child, Parent};
use yew::prelude::*;
use yewdux::prelude::*;

const SWIFT_SYNTAX: &str = include_str!("../syntaxes/Swift.sublime-syntax");

mod raw_html;
mod theme;

struct App {
    link: ComponentLink<Self>,
    rust_textarea_value: String,

    theme_handler: PageThemeHandler,

    // Syntax stuff
    ss: SyntaxSet,
    theme_set: ThemeSet,
}

enum Msg {
    TextAreaCallback(String),
    PageTheme(Rc<PageTheme>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let theme_handler = PageThemeHandler {
            page_theme: Default::default(),
            page_theme_dispatch: Dispatch::bridge_state(link.callback(Msg::PageTheme)),
        };

        let mut ss = SyntaxSet::load_defaults_newlines().into_builder();
        ss.add(SyntaxDefinition::load_from_str(SWIFT_SYNTAX, true, Some("swift")).unwrap());
        let ss = ss.build();
        let theme_set = ThemeSet::load_defaults();

        Self {
            link,
            rust_textarea_value: "".to_string(),
            theme_handler,
            ss,
            theme_set,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::TextAreaCallback(value) => {
                self.rust_textarea_value = value;
                true
            }
            Msg::PageTheme(page_theme) => {
                theme::set_page_theme(*page_theme);
                self.theme_handler.page_theme = page_theme;
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let text_area_value = &self.rust_textarea_value;
        let out = {
            let mut out_ts: Vec<u8> = Vec::new();
            let _ = process_input(text_area_value, &TypeScript::default(), &mut out_ts);

            let mut out_kotlin: Vec<u8> = Vec::new();
            let _ = process_input(text_area_value, &Kotlin::default(), &mut out_kotlin);

            let mut out_swift: Vec<u8> = Vec::new();
            let _ = process_input(text_area_value, &Swift::default(), &mut out_swift);

            (out_ts, out_kotlin, out_swift)
        };

        let output_theme = match &self.theme_handler.page_theme.as_ref() {
            PageTheme::Dark => "base16-eighties.dark",
            PageTheme::Light => "base16-ocean.light",
        };

        let typescript_output = highlighted_html_for_string(
            &String::from_utf8(out.0).unwrap(),
            &self.ss,
            self.ss.find_syntax_by_token("js").unwrap(),
            &self.theme_set.themes[output_theme],
        );
        let kotlin_output = highlighted_html_for_string(
            &String::from_utf8(out.1).unwrap(),
            &self.ss,
            self.ss.find_syntax_by_token("java").unwrap(),
            &self.theme_set.themes[output_theme],
        );
        let swift_output = highlighted_html_for_string(
            &String::from_utf8(out.2).unwrap(),
            &self.ss,
            self.ss.find_syntax_by_token("swift").unwrap(),
            &self.theme_set.themes[output_theme],
        );

        let page_theme = self.theme_handler.clone();

        html! {
            <>
            <ybc::Navbar
                classes= classes!("is-success")
                padded=true
                navbrand=html!{
                    <>
                        <ybc::NavbarItem>
                            <img src="1plogo.png"/>
                        </ybc::NavbarItem>
                        <ybc::NavbarItem>
                            <ybc::Title classes=classes!("has-text-white") size=ybc::HeaderSize::Is4>
                                {"Typeshare REPL"}
                            </ybc::Title>
                        </ybc::NavbarItem>
                    </>
                }
                navstart=html!{}
                navend=html!{
                    <ybc::NavbarItem>
                        <theme::Toggle with page_theme />
                    </ybc::NavbarItem>
                }
            />

            <ybc::Container fluid=true>
                <ybc::Tile ctx=Ancestor>
                    <ybc::Tile ctx=Parent vertical=true size=ybc::TileSize::Six>
                        <ybc::Tile ctx=Child>
                            <ybc::TextArea
                                name="rust"
                                placeholder="Rust source input here..."
                                value={self.rust_textarea_value.clone()}
                                rows=12
                                update=self.link.callback(|v| Msg::TextAreaCallback(v))
                            ></ybc::TextArea>
                        </ybc::Tile>
                        <ybc::Tile ctx=Child classes=classes!("code-output")>
                            <RawHtml inner_html={swift_output}></RawHtml>
                        </ybc::Tile>
                    </ybc::Tile>
                    <ybc::Tile ctx=Parent vertical=true>
                        <ybc::Tile ctx=Child classes=classes!("code-output")>
                            <RawHtml inner_html={typescript_output}></RawHtml>
                        </ybc::Tile>
                        <ybc::Tile ctx=Child classes=classes!("code-output")>
                            <RawHtml inner_html={kotlin_output}></RawHtml>
                        </ybc::Tile>
                    </ybc::Tile>
                </ybc::Tile>
            </ybc::Container>
            </>
        }
    }
}

fn main() {
    set_panic_hook();
    yew::start_app::<App>();
}

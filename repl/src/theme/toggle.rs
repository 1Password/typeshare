use super::*;

pub struct Toggle {
    theme_handler: PageThemeHandler,
}

impl Component for Toggle {
    type Message = ();
    type Properties = PageThemeHandler;

    fn create(theme_handler: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { theme_handler }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, theme_handler: Self::Properties) -> ShouldRender {
        if self.theme_handler.page_theme != theme_handler.page_theme {
            self.theme_handler = theme_handler;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let button_text = match self.theme_handler.page_theme.as_ref() {
            PageTheme::Dark => "Light Mode 🙈",
            PageTheme::Light => "Dark Mode 😎",
        };
        let toggle = self
            .theme_handler
            .page_theme_dispatch
            .reduce_callback(PageTheme::toggle);

        html! {
            <ybc::Button onclick={toggle}>{button_text}</ybc::Button>
        }
    }
}

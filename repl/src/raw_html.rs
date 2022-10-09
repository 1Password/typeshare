//! A small component to allow for inserting raw HTML into the DOM.
//!
//! See https://github.com/yewstack/yew/issues/189

use yew::{prelude::*, virtual_dom::VNode, web_sys::Node};

#[derive(Debug, Clone, Eq, PartialEq, Properties)]
pub struct RawHtmlProps {
    pub inner_html: String,
}

pub struct RawHtml {
    props: RawHtmlProps,
}

impl Component for RawHtml {
    type Message = ();
    type Properties = RawHtmlProps;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let div = yew::utils::document().create_element("div").unwrap();
        div.set_inner_html(&self.props.inner_html[..]);

        let node = Node::from(div);
        VNode::VRef(node)
    }
}

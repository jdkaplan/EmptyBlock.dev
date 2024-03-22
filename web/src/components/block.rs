use yew::prelude::*;

use crate::apps::mosaic::Rgba;

#[derive(Properties, PartialEq, Debug)]
pub struct BlockProps {
    #[prop_or_default]
    pub class: Classes,

    pub background: Rgba,
    pub foreground: Rgba,
}

#[function_component]
pub fn Block(props: &BlockProps) -> Html {
    let bg = style("background-color", &props.background.css());
    let fg = style("background-color", &props.foreground.css());

    html! {
        <div class={classes!("h-full", "w-full", "aspect-square", props.class.clone())} style={bg}>
            <div class="h-full w-full" style={fg} />
        </div>
    }
}

fn style(property: &str, value: &str) -> String {
    format!("{}: {};", property, value)
}

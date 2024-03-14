use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    #[prop_or_default]
    pub children: Html,
    pub error: String,
}

#[function_component]
pub fn Error(props: &ErrorProps) -> Html {
    html! {
        <div class="alert">
            { props.children.clone() }
            <pre>{props.error.clone()}</pre>
        </div>
    }
}

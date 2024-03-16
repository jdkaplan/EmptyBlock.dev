use yew::prelude::*;

use crate::Route;

type Link = yew_router::components::Link<Route>;

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn Header(props: &HeaderProps) -> Html {
    html! {
        <nav class="flex flex-row justify-between bg-gray-200 dark:bg-gray-800 px-3 py-1">
            <Link to={Route::Home}>{"EmptyBlock.dev"}</Link>

            <div class="space-x-4">
                { props.children.clone() }
            </div>
        </nav>
    }
}

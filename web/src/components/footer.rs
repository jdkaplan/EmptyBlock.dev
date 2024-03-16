use yew::prelude::*;

#[function_component]
pub fn Footer() -> Html {
    html! {
        <footer class="flex flex-row justify-between bg-gray-200 dark:bg-gray-800 px-3 py-1">
            <a href="/about">{"About"}</a>
        </footer>
    }
}

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::*;
use crate::hooks::*;
use crate::Route;

type Link = yew_router::components::Link<Route>;

#[function_component]
pub fn Trellis() -> Html {
    use_title("Trellis");
    let config_ctx = use_context::<TrellisConfigContext>().unwrap();

    let inner = match &config_ctx.inner {
        Ok(config) => html! { <Board config={config.clone()} class="flex-grow" /> },
        Err(err) => html! {
            <Error error={err.clone()}>
                <p>{"Could not load Trellis config. The error details should be in the console log."}</p>
            </Error>
        },
    };

    html! {
        <div class="min-h-screen flex flex-col text-black bg-white dark:text-white dark:bg-black">
            <nav class="flex flex-row justify-between bg-gray-200 dark:bg-gray-900 px-3 py-1">
                <a href={crate::Route::Home.to_path()}>{"EmptyBlock.dev"}</a>
                <div class="space-x-4">
                    <Link to={Route::TrellisConfig}>{"Board Settings"}</Link>
                    <span>{"TODO(login)"}</span>
                </div>
            </nav>

            <main class="flex-grow flex flex-col justify-between m-1">
                {inner}
            </main>

            <footer class="flex flex-row justify-between">
                <p class="text-small">{"someone please help me style this 😅"}</p>
            </footer>
        </div>
    }
}

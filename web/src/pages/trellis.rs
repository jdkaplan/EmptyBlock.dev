use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::apps::trellis::Config;
use crate::components::*;
use crate::hooks::*;
use crate::Route;

type Link = yew_router::components::Link<Route>;

const LOCAL_STORAGE_KEY: &str = "trellis.config";

#[function_component]
pub fn Trellis() -> Html {
    use_title("Trellis");

    // TODO(users): Sync LocalStorage with the DB
    let config = use_state(|| match LocalStorage::get(LOCAL_STORAGE_KEY) {
        Ok(config) => Ok(config),
        Err(err @ StorageError::KeyNotFound(_)) => {
            tracing::info!({ ?err }, "No Trellis config found. Using starter config");
            Ok(Config::starter())
        }
        Err(err) => {
            let value = LocalStorage::raw().get_item(LOCAL_STORAGE_KEY);
            tracing::error!({ ?err, ?value }, "Could not parse Trellis config");
            Err(err)
        }
    });

    let inner = match &*config {
        Ok(config) => html! { <Board config={config.clone()} class="flex-grow" /> },
        Err(err) => html! {
            <div class="alert">
                <p>{"Could not load Trellis config. The error details should be in the console log."}</p>
                <pre>{err.to_string()}</pre>
            </div>
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
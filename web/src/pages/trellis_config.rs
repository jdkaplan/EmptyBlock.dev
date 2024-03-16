use yew::prelude::*;
use yew_router::prelude::*;

use crate::apps::trellis::Config;
use crate::components::*;
use crate::hooks::*;
use crate::Route;

#[function_component]
pub fn TrellisConfig() -> Html {
    use_title("Trellis");
    let navigator = use_navigator().unwrap();
    let config_ctx = use_context::<TrellisConfigContext>().unwrap();

    let inner = match &config_ctx.inner {
        Ok(config) => {
            let onsubmit = {
                let config_ctx = config_ctx.clone();
                Callback::from(move |config: Option<Config>| {
                    tracing::debug!({ ?config }, "Editor result");

                    if let Some(config) = config {
                        config_ctx.dispatch(TrellisConfigAction::Save(config))
                    };

                    navigator.push(&Route::Trellis);
                })
            };

            html! {
                <BoardEditor config={config.clone()} class="flex-grow" {onsubmit} />
            }
        }
        Err(err) => html! {
            <Error error={err.clone()}>
                <p>{"Could not load Trellis config. The error details should be in the console log."}</p>
            </Error>
        },
    };

    html! {
        <div class="min-h-screen flex flex-col text-black bg-white dark:text-white dark:bg-black">
            <Header>
                <span>{"TODO(login)"}</span>
            </Header>

            <main class="flex-grow flex flex-col justify-between m-1">
                {inner}
            </main>

            <Footer />
        </div>
    }
}

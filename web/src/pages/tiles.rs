use eyre::WrapErr;
use gloo::history::{BrowserHistory, History};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::apps::tiles::Module;
use crate::components::*;
use crate::hooks::*;
use crate::Route;

#[derive(Clone, PartialEq, Default, Deserialize, Serialize)]
struct Query {
    seed: Option<u64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ViewState {
    Run,
    Edit,
}

#[function_component]
pub fn Tiles() -> Html {
    use_title("Tiles");

    let location = use_location().unwrap();
    let history = BrowserHistory::new();
    let view_state = use_state(|| ViewState::Run);

    let update = use_state(|| Module::decode(location.hash()).unwrap_or_default());

    let seed = use_state(|| {
        let query = location.query::<Query>().unwrap_or_default();
        query.seed.unwrap_or_else(rand::random)
    });

    use_effect_with((), {
        let history = history.clone();
        let update = update.clone();
        let seed = seed.clone();
        move |_| {
            replace_url(&history, &update, *seed).unwrap();
        }
    });

    use_body_class(vec![
        "flex",
        "flex-row",
        "h-screen",
        "w-screen",
        "items-center",
        "justify-center",
    ]);

    tracing::debug!({ ?seed }, "Tiles");
    tracing::debug!("\n{}", *update);

    let show_editor = {
        let view_state = view_state.clone();
        Callback::from(move |_| view_state.set(ViewState::Edit))
    };

    let onsubmit = {
        let view_state = view_state.clone();

        Callback::from(move |val: Option<SimulationEditorValue>| {
            tracing::debug!({ ?val }, "Editor result");

            if let Some(val) = val {
                push_url(&history, &val.module, val.seed).unwrap();
            };

            view_state.set(ViewState::Run);
        })
    };

    let inner = match *view_state {
        ViewState::Run => html! {
            <Simulation update={update.binary.clone()} seed={*seed} />
        },
        ViewState::Edit => html! {
            <SimulationEditor source={update.text.clone()} seed={*seed} onsubmit={onsubmit} class="p-1" />
        },
    };

    let right_nav = match *view_state {
        ViewState::Run => html! {
            <div>
                <button onclick={show_editor}>{"Edit"}</button>
            </div>
        },
        ViewState::Edit => html! {},
    };

    // I haven't been able to get this layout to be exactly what I want. Ideally, this is a
    // full-screen app (never scrolls, fills as much space as possible) _and_ all the simulation's
    // grid cells are square.
    //
    // So far the best I can get is that the whole app is square and fills the screen, which makes
    // the grid cells a little wider than they are tall. I don't want to _remove_ the header and
    // footer, so I'm giving up for now.
    //
    // TODO: There must be some way to satisfy both "full screen app" and "square grid cells"....

    html! {
        <div class="flex flex-col items-stretch justify-between h-full w-full">
            <nav class="flex flex-row justify-between">
                <a href={crate::Route::Home.to_path()}>{"EmptyBlock.dev"}</a>
                { right_nav }
            </nav>

            {inner}

            <footer class="flex flex-row justify-between">
                <p class="text-small">{"someone please help me style this 😅"}</p>
            </footer>
        </div>
    }
}

fn replace_url(history: &BrowserHistory, module: &Module, seed: u64) -> eyre::Result<()> {
    history
        .replace_with_query(
            format!("{}#{}", Route::Tiles.to_path(), module.encode()),
            Query { seed: Some(seed) },
        )
        .wrap_err("replace history")
}

fn push_url(history: &BrowserHistory, module: &Module, seed: u64) -> eyre::Result<()> {
    history
        .push_with_query(
            format!("{}#{}", Route::Tiles.to_path(), module.encode()),
            Query { seed: Some(seed) },
        )
        .wrap_err("replace history")
}

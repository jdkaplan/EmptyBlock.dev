use eyre::WrapErr;
use gloo::history::{BrowserHistory, History};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::apps::mosaic::Module;
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
pub fn Mosaic() -> Html {
    use_title("Mosaic");

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

    use_body_class(vec!["h-screen", "w-screen"]);

    tracing::debug!({ ?seed }, "Mosaic");
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
            <div class="flex justify-center items-center h-full container-size">
                <div class="box-square">
                    <Simulation update={update.binary.clone()} seed={*seed} class="h-full w-full"/>
                </div>
            </div>
        },
        ViewState::Edit => html! {
            <SimulationEditor source={update.text.clone()} seed={*seed} onsubmit={onsubmit} class="p-1" />
        },
    };

    html! {
        <div class="flex flex-col justify-between h-full w-full">
            <Header>
                if *view_state == ViewState::Run {
                    <button onclick={show_editor}>{"Edit"}</button>
                }
            </Header>

            {inner}

            <Footer />
        </div>
    }
}

fn replace_url(history: &BrowserHistory, module: &Module, seed: u64) -> eyre::Result<()> {
    history
        .replace_with_query(
            format!("{}#{}", Route::Mosaic.to_path(), module.encode()),
            Query { seed: Some(seed) },
        )
        .wrap_err("replace history")
}

fn push_url(history: &BrowserHistory, module: &Module, seed: u64) -> eyre::Result<()> {
    history
        .push_with_query(
            format!("{}#{}", Route::Mosaic.to_path(), module.encode()),
            Query { seed: Some(seed) },
        )
        .wrap_err("replace history")
}

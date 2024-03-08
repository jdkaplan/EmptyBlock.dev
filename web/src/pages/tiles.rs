use gloo::utils::{body, document};
use serde::Deserialize;
use web_sys::js_sys::Array;
use web_sys::wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::apps::tiles::Module;
use crate::components::*;

#[derive(Clone, PartialEq, Default, Deserialize)]
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
    let location = use_location().unwrap();
    let view_state = use_state(|| ViewState::Run);

    let update = use_state(|| {
        Module::decode(location.hash()).unwrap_or_else(|| {
            let src = Module::default();
            set_hash(&src);
            src
        })
    });

    let seed = use_state(|| {
        let query = location.query::<Query>().unwrap_or_default();
        query.seed.unwrap_or_else(rand::random)
    });

    use_effect(move || {
        let title = document().title();
        document().set_title("Tiles");

        let classes = vec![
            "flex",
            "flex-row",
            "h-screen",
            "w-screen",
            "items-center",
            "justify-center",
        ]
        .into_iter()
        .map(JsValue::from_str)
        .collect::<Array>();

        body().class_list().add(&classes).unwrap();

        move || {
            document().set_title(&title);
            body().class_list().remove(&classes).unwrap();
        }
    });

    tracing::debug!({ ?seed }, "Tiles");
    tracing::debug!("\n{}", *update);

    let show_editor = {
        let view_state = view_state.clone();
        Callback::from(move |_| view_state.set(ViewState::Edit))
    };

    let onsubmit = {
        let view_state = view_state.clone();
        let update = update.clone();
        let seed = seed.clone();

        Callback::from(move |val: Option<EditorValue>| {
            tracing::debug!({ ?val }, "Editor result");

            if let Some(val) = val {
                set_hash(&val.module);
                set_query(val.seed);

                update.set(val.module);
                seed.set(val.seed);
            };

            view_state.set(ViewState::Run);
        })
    };

    let inner = match *view_state {
        ViewState::Run => html! {
            <Simulation update={update.binary.clone()} seed={*seed} />
        },
        ViewState::Edit => html! {
            <Editor source={update.text.clone()} seed={*seed} onsubmit={onsubmit} class="p-1" />
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

fn set_query(seed: u64) {
    let location = document().location().expect("browser has a location");
    location
        .set_search(&format!("seed={}", seed))
        .expect("query is mutable");
}

fn set_hash(module: &Module) {
    let location = document().location().expect("browser has a location");
    location
        .set_hash(&module.encode())
        .expect("hash is mutable");
}

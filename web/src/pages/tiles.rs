use base64::Engine as _;
use gloo::utils::{body, document};
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::*;

const DEFAULT_UPDATE: &str = include_str!("../data/default.wat");

const BASE64_URL_SAFE_LENIENT: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::URL_SAFE,
    base64::engine::GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
);

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

        body()
            .class_list()
            .add_4("flex-row", "fullscreen", "align-center", "justify-center")
            .unwrap();

        move || {
            document().set_title(&title);
            body()
                .class_list()
                .remove_4("flex-row", "fullscreen", "align-center", "justify-center")
                .unwrap();
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
                let module = Module::new(val.source).unwrap();
                set_hash(&module);
                set_query(val.seed);

                update.set(module);
                seed.set(val.seed);
            };

            view_state.set(ViewState::Run);
        })
    };

    let inner = match *view_state {
        ViewState::Run => html! {
            <Simulation update={update.binary.clone()} seed={*seed} class={classes!("square", "min-0")} />
        },
        ViewState::Edit => html! {
            <Editor source={update.text.clone()} seed={*seed} onsubmit={onsubmit} class={classes!("flex-col")} />
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
        <div class={classes!("full-aspect", "flex-col", "align-stretch", "justify-between")}>
            <nav class={classes!("flex-row", "justify-between")}>
                <a href={crate::Route::Home.to_path()}>{"EmptyBlock.dev"}</a>
                { right_nav }
            </nav>

            {inner}

            <footer class={classes!("flex-row", "justify-between")}>
                <p class="text-small">{"someone please help me style this 😅"}</p>
                <a target="_blank" href="/about">{"About"}</a>
            </footer>
        </div>
    }
}

#[derive(Debug, Clone)]
struct Module {
    text: String,
    binary: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
enum DecodeModuleError {
    #[error("empty string")]
    Empty,

    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("invalid WAT: {0}")]
    Wat(#[from] wat::Error),
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Default for Module {
    fn default() -> Self {
        let text = String::from(DEFAULT_UPDATE);
        let binary = wat::parse_str(&text).expect("default WAT is valid");
        Self { text, binary }
    }
}

impl Module {
    fn new(text: String) -> Result<Self, wat::Error> {
        let binary = wat::parse_str(&text)?;
        Ok(Self { text, binary })
    }

    fn decode(hash: &str) -> Option<Self> {
        match Self::try_decode(hash) {
            Ok(v) => Some(v),
            Err(DecodeModuleError::Empty) => None,
            Err(err) => {
                tracing::error!({ ?hash, ?err }, "invalid URL hash");
                None
            }
        }
    }

    fn try_decode(hash: &str) -> Result<Self, DecodeModuleError> {
        // Remove the leading hash character (#) for convenience.
        let hash = hash.trim_start_matches('#');
        if hash.is_empty() {
            return Err(DecodeModuleError::Empty);
        }

        let decoded = BASE64_URL_SAFE_LENIENT.decode(hash)?;
        let text = String::from_utf8(decoded)?;
        let binary = wat::parse_str(&text)?;
        Ok(Self { text, binary })
    }

    fn encode(&self) -> String {
        BASE64_URL_SAFE_LENIENT.encode(&self.text)
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

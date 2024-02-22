use gloo::net::http::Request;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[cfg(debug_assertions)]
const TRACING_LEVEL: tracing::Level = tracing::Level::DEBUG;

#[cfg(not(debug_assertions))]
const TRACING_LEVEL: tracing::Level = tracing::Level::INFO;

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(tracing_web::MakeConsoleWriter)
        .with_filter(LevelFilter::from_level(TRACING_LEVEL));

    let perf_layer = tracing_web::performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let msg = use_state(|| String::from("Loading data..."));
    {
        let msg = msg.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match fetch_greeting().await {
                    Ok(greeting) => msg.set(greeting),
                    Err(err) => tracing::error!("Could not fetch greeting: {:?}", err),
                }
            })
        })
    }

    html! {
        <h1>{ &*msg }</h1>
    }
}

async fn fetch_greeting() -> eyre::Result<String> {
    let res = Request::get("/api/hello").send().await?;
    if !res.ok() {
        return Err(eyre::eyre!("error response: {:?}", res));
    }

    let text = res.text().await?;
    Ok(text)
}

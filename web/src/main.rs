use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::prelude::*;
use yew::prelude::*;
use yew::suspense::use_future;
use yew_router::prelude::*;

mod apps;
mod components;
mod hooks;
mod pages;
mod types;

use crate::types::Session;

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

// TODO: Make dedicated edit routes for forms
#[derive(Clone, PartialEq, Routable)]
enum Route {
    #[not_found]
    #[at("/404")]
    NotFound,

    #[at("/")]
    Home,

    #[at("/mosaic")]
    Mosaic,

    #[at("/trellis")]
    Trellis,

    #[at("/trellis/config")]
    TrellisConfig,
}

impl Route {
    fn render(self) -> Html {
        match self {
            Route::NotFound => html! { <pages::NotFound /> },
            Route::Home => html! { <pages::Home /> },
            Route::Mosaic => html! { <pages::Mosaic /> },
            Route::Trellis => html! {
                <components::TrellisConfigProvider>
                    <pages::Trellis />
                </components::TrellisConfigProvider>
            },
            Route::TrellisConfig => html! {
                <components::TrellisConfigProvider>
                    <pages::TrellisConfig />
                </components::TrellisConfigProvider>
            },
        }
    }
}

#[function_component]
fn App() -> Html {
    let loading = html! { <p>{"Loading app..."}</p> };

    html! {
        <Suspense fallback={loading}>
            <WaitForSession>
                <BrowserRouter>
                    <Switch<Route> render={Route::render} />
                </BrowserRouter>
            </WaitForSession>
        </Suspense>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
fn WaitForSession(props: &Props) -> HtmlResult {
    let session = use_future(Session::load_ok)?;

    Ok(html! {
        <ContextProvider<Option<Session>> context={(*session).clone()}>
            { props.children.clone () }
        </ContextProvider<Option<Session>>>
    })
}

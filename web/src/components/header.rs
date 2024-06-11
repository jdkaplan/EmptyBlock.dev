use gloo::net::http::Request;
use gloo::utils::document;
use wasm_bindgen_futures::spawn_local;
use web_sys::Location;
use yew::prelude::*;

use crate::types::Session;
use crate::Route;

type Link = yew_router::components::Link<Route>;

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn Header(props: &HeaderProps) -> Html {
    let session = use_context::<Option<Session>>().unwrap();

    let log_in_out = match session {
        Some(session) => html! {
            <button
                type="button"
                onclick={Callback::from(move |_| spawn_local(log_out(session.clone())))}
            >{"Log out"}</button>
        },

        None => html! {
            <a href="/oauth/start">{"Log in"}</a>
        },
    };

    html! {
        <nav class="flex flex-row justify-between bg-gray-200 dark:bg-gray-800 px-3 py-1">
            <Link to={Route::Home}>{"EmptyBlock.dev"}</Link>

            <div class="space-x-4">
                { props.children.clone() }

                { log_in_out }
            </div>
        </nav>
    }
}

const CSRF_TOKEN_HEADER: &str = "X-Csrf-Token";

async fn log_out(session: Session) {
    let location = document().location().expect("page always has location");

    if let Err(res) = Request::delete("/session")
        .header(CSRF_TOKEN_HEADER, session.csrf_token.secret())
        .send()
        .await
    {
        tracing::error!({ ?res }, "logout request failed");
    }

    location.reload().expect("same-origin always succeeds");
}

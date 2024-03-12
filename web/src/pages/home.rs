use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::Route;

type Link = yew_router::components::Link<Route>;

#[function_component]
pub fn Home() -> Html {
    let msg = use_state(|| String::from("Loading greeting..."));
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
        <main>
            <h1>{"EmptyBlock.dev"}</h1>

            <p class="text-green-600">{ &*msg }</p>

            <ul>
                <li><Link to={Route::Home}>{"Home"}</Link></li>
                <li><Link to={Route::Tiles}>{"Tiles"}</Link></li>
                <li><Link to={Route::Trellis}>{"Trellis"}</Link></li>
                <li><a href="/about">{"About"}</a></li>
            </ul>
        </main>
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

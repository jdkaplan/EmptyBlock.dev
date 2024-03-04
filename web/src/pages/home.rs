use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component]
pub fn Home() -> Html {
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
        <h1 class="greeting">{ &*msg }</h1>
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

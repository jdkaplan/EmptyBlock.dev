use gloo::net::http::Request;
use yew::prelude::*;
use yew::suspense::use_future;

use crate::Route;

type Link = yew_router::components::Link<Route>;

#[function_component]
pub fn Home() -> Html {
    let fallback_greeting = html! { <p>{"Loading greeting..."}</p> };

    html! {
        <main>
            <h1>{"EmptyBlock.dev"}</h1>

            <Suspense fallback={fallback_greeting}>
                <Greeting />
            </Suspense>

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

#[function_component]
fn Greeting() -> HtmlResult {
    let greeting = use_future(fetch_greeting)?;

    match &*greeting {
        Ok(greeting) => Ok(html! {
            <p class="text-green-600 dark:text-green-400">{greeting}</p>
        }),
        Err(err) => Ok(html! {
            <div class="alert">
                <p>{"Could not fetch greeting:"}</p>
                <pre>{err.to_string()}</pre>
            </div>
        }),
    }
}

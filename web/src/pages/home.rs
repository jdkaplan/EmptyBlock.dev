use gloo::net::http::Request;
use yew::prelude::*;
use yew::suspense::use_future;

use crate::components::*;
use crate::Route;

type Link = yew_router::components::Link<Route>;

#[function_component]
pub fn Home() -> Html {
    let fallback_greeting = html! { <p>{"Loading greeting..."}</p> };

    html! { <>
        <Header />

        <main class="m-3 max-w-prose">
            <h1>{"EmptyBlock.dev"}</h1>

            <p>{"A playground for web app experiments"}</p>

            <Suspense fallback={fallback_greeting}>
                <Greeting />
            </Suspense>

            <ul class="list-bulleted">
                <li><Link to={Route::Mosaic}>{"Mosaic"}</Link></li>
                <li><Link to={Route::Trellis}>{"Trellis"}</Link></li>
                <li>{"(your experiment?)"}</li>
            </ul>
        </main>

        <Footer />
    </> }
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

async fn fetch_greeting() -> eyre::Result<String> {
    let res = Request::get("/api/hello").send().await?;
    if !res.ok() {
        return Err(eyre::eyre!("error response: {:?}", res));
    }

    let text = res.text().await?;
    Ok(text)
}

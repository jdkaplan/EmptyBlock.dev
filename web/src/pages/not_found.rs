use yew::prelude::*;

use crate::components::*;
use crate::Route;

type Link = yew_router::components::Link<Route>;

#[function_component]
pub fn NotFound() -> Html {
    html! { <>
        <Header>
            <span>{"TODO(login)"}</span>
        </Header>

        <main class="m-3 max-w-prose">
            <h1>{"Not Found"}</h1>

            <p>{"There's nothing here!"}</p>

            <p>{"If you think there should have been something at this route, you're probably right. Please let me know about it!"}</p>

            <p><Link to={Route::Home}>{"Back to Home"}</Link></p>
        </main>

        <Footer />
    </> }
}

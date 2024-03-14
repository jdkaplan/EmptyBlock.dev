use uuid::Uuid;
use yew::prelude::*;

use crate::apps::trellis;

#[derive(Properties, PartialEq)]
pub struct CounterProps {
    #[prop_or_default]
    pub value: i64,

    #[prop_or_default]
    pub onchange: Callback<i64>,
}

#[function_component]
pub fn Counter(props: &CounterProps) -> Html {
    let onclick = {
        let onchange = props.onchange.clone();
        move |_| onchange.emit(1)
    };

    html! {
        <div class="flex flex-col items-stretch justify-around w-full h-full">
            <div class="text-7xl text-center">{ props.value }</div>
            <button class="text-xl" {onclick}>{ "+1" }</button>
        </div>
    }
}

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CounterProps {
    #[prop_or_default]
    pub start: i64,
}

#[function_component]
pub fn Counter(props: &CounterProps) -> Html {
    let count = use_state(|| props.start);
    let onclick = {
        let count = count.clone();
        move |_| count.set(*count + 1)
    };

    html! {
        <div class="flex flex-col items-stretch justify-around w-full h-full">
            <div class="text-7xl text-center">{ *count }</div>
            <button class="text-xl" {onclick}>{ "+1" }</button>
        </div>
    }
}

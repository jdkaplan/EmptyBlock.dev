use yew::prelude::*;

use crate::apps::tiles::{Blocks, GRID_SIZE};

use crate::components::*;

#[derive(Properties, PartialEq, Debug)]
pub struct GridProps {
    #[prop_or_default]
    pub class: Classes,

    pub prev: Blocks,
    pub next: Blocks,
}

#[function_component]
pub fn Grid(props: &GridProps) -> Html {
    let mut children: Vec<Html> = Vec::with_capacity(GRID_SIZE * GRID_SIZE);

    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            let key = r * GRID_SIZE + c;

            let background = props.prev[(r, c)];
            let foreground = props.next[(r, c)];

            children.push(html! {
                <Block key={key} background={background} foreground={foreground} class={classes!("square", "min-1")} />
            });
        }
    }

    html! {
        <div class={classes!("grid", props.class.clone())}>
            { children }
        </div>
    }
}

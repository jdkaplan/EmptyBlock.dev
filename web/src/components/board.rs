use yew::prelude::*;

use crate::apps::trellis::{Config, Data};
use crate::components::*;

#[derive(Properties, PartialEq, Debug)]
pub struct BoardProps {
    #[prop_or_default]
    pub class: Classes,

    pub config: Config,
}

#[function_component]
pub fn Board(props: &BoardProps) -> Html {
    let class = classes!(
        "grid",
        "grid-cols-1",
        "md:grid-cols-2",
        "lg:grid-cols-3",
        "xl:grid-cols-4",
        "gap-1",
        "auto-rows-fr",
        props.class.clone(),
    );

    html! {
        <div {class}>
            { for render_tiles(&props.config) }
        </div>
    }
}

fn render_tiles(config: &Config) -> Vec<Html> {
    let mut children = Vec::new();

    for tile in &config.layout.tiles {
        let child = match &tile.data {
            Data::Clock => html! { <Clock /> },
            Data::Weather(weather) => {
                let location_id = weather.location_id.clone().unwrap_or_default();
                let owm_api_key = config.secrets.open_weather.clone().unwrap_or_default();
                html! { <Weather {location_id} {owm_api_key} /> }
            }
            Data::Note(note) => {
                let initial = note.text.clone();
                // TODO: Use onchange to save data
                html! { <Note {initial} />}
            }
            Data::Counter(counter) => {
                let start = counter.value;
                // TODO: Use onchange to save data
                html! { <Counter {start} />}
            }
        };
        children.push(child);
    }

    children
}

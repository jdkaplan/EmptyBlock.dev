use yew::prelude::*;

use crate::apps::trellis;
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
    let config_ctx = use_context::<TrellisConfigContext>().unwrap();

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
            { for render_tiles(config_ctx) }
        </div>
    }
}

fn render_tiles(config_ctx: TrellisConfigContext) -> Vec<Html> {
    let config = config_ctx.inner.clone().unwrap();
    let mut children = Vec::new();

    for tile in &config.layout.tiles {
        let id = tile.id;
        let config_ctx = config_ctx.clone();

        let child = match &tile.data {
            Data::Clock => html! { <Clock /> },
            Data::Weather(weather) => {
                let location_id = weather.location_id.clone().unwrap_or_default();
                let owm_api_key = config.secrets.open_weather.clone().unwrap_or_default();
                html! { <Weather {location_id} {owm_api_key} /> }
            }
            Data::Note(note) => {
                let initial = note.text.clone();
                let onchange = Callback::from(move |text| {
                    config_ctx.dispatch(TrellisConfigAction::Update {
                        id,
                        data: Data::Note(trellis::Note {
                            // BUG: This overwrites changes from other sources.
                            // TODO: Build this update out of reducers too?
                            text,
                        }),
                    });
                });

                html! { <Note {initial} {onchange} />}
            }
            Data::Counter(counter) => {
                let value = counter.value;

                let onchange = {
                    Callback::from(move |delta| {
                        config_ctx.dispatch(TrellisConfigAction::Update {
                            id,
                            data: Data::Counter(trellis::Counter {
                                // BUG: This overwrites changes from other sources.
                                // TODO: Build this update out of reducers too?
                                value: value + delta,
                            }),
                        });
                    })
                };

                html! { <Counter {value} {onchange} />}
            }
        };
        children.push(child);
    }

    children
}

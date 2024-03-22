use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::apps::mosaic::Module;

#[derive(Properties, PartialEq, Debug)]
pub struct SimulationEditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub seed: u64,
    pub source: String,

    pub onsubmit: Callback<Option<SimulationEditorValue>>,
}

#[derive(PartialEq, Debug)]
pub struct SimulationEditorValue {
    pub seed: u64,
    pub module: Module,
}

pub struct SimulationEditor {
    seed_ref: NodeRef,
    source_ref: NodeRef,

    error: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimulationEditorMsg {
    Save,
    Cancel,
}

impl Component for SimulationEditor {
    type Message = SimulationEditorMsg;

    type Properties = SimulationEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            seed_ref: NodeRef::default(),
            source_ref: NodeRef::default(),
            error: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onsubmit = ctx.link().callback(|_| SimulationEditorMsg::Save);
        let oncancel = ctx.link().callback(|_| SimulationEditorMsg::Cancel);

        // TODO: Explain the module requirements

        html! {
            <div class={classes!("flex", "flex-col", props.class.clone())}>
                <div class="max-w-prose">{docs()}</div>

                <form action="javascript:void(0);" class="flex flex-col">
                    if let Some(err) = &self.error {
                        <div class="alert"><pre>{err.to_string()}</pre></div>
                    }

                    <label for="seed">{"Seed"}</label>
                    <input name="seed" ref={self.seed_ref.clone()} type="text" />

                    <label for="source">{"Module"}</label>
                    <textarea name="source" ref={self.source_ref.clone()} />

                    <div class="flex flex-row justify-around">
                        <button type="button" onclick={oncancel}>{"Cancel"}</button>
                        <button type="button" onclick={onsubmit}>{"Submit"}</button>
                    </div>
                </form>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            let rows = 1 + props.source.lines().count() as u32;

            let seed = self.seed_ref.cast::<HtmlInputElement>().unwrap();
            let source = self.source_ref.cast::<HtmlTextAreaElement>().unwrap();

            seed.set_value(&props.seed.to_string());
            source.set_value(&props.source);
            source.set_rows(rows);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            SimulationEditorMsg::Save => {
                let seed = self.seed_ref.cast::<HtmlInputElement>().unwrap();
                let source = self.source_ref.cast::<HtmlTextAreaElement>().unwrap();

                match Module::new(source.value()) {
                    Ok(module) => {
                        props.onsubmit.emit(Some(SimulationEditorValue {
                            seed: seed.value().parse().unwrap_or_default(),
                            module,
                        }));
                        false
                    }
                    Err(err) => {
                        self.error = Some(err.to_string());
                        true
                    }
                }
            }
            SimulationEditorMsg::Cancel => {
                props.onsubmit.emit(None);
                false
            }
        }
    }
}

fn docs() -> Html {
    html! { <>
    <p>{"The "}<code>{"next"}</code>{" function will be called for each cell in the grid to populate the grid for each tick."}</p>
    <p>{"The parameters are the cell's neighborhood values from the previous tick in row-major order. For example, "}<code>{"$p00"}</code>{" is the upper-left cell, "}<code>{"$p22"}</code>{" is the lower-right, and "}<code>{"$p11"}</code>{" is the value of the current cell."}</p>

    <p>{"The bits of each "}<code>{"i32"}</code>{" are packed as RGBA (8 bits for each channel)."}</p>

    <p>{"This WebAssembly Text format (WAT) isn't really meant for authoring code, but it "}<em>{"is"}</em>{" described in "}<a href="https://webassembly.github.io/spec/core/text/index.html">{"the spec"}</a>{"."}</p>
    </> }
}

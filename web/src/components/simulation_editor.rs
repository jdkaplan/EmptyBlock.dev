use eyre::OptionExt;
use gloo::timers::callback::Timeout;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::apps::mosaic::{Blocks, Module};
use crate::components::*;

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

    preview: Blocks,
    pending_update: Option<Timeout>,

    error: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SimulationEditorMsg {
    EditSeed,
    UpdatePreview,
    Save,
    Cancel,
}

impl Component for SimulationEditor {
    type Message = SimulationEditorMsg;

    type Properties = SimulationEditorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();

        Self {
            seed_ref: NodeRef::default(),
            source_ref: NodeRef::default(),
            preview: Blocks::default(),
            pending_update: Some(Timeout::new(0, move || {
                link.send_message(SimulationEditorMsg::UpdatePreview)
            })),
            error: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let oninput = ctx.link().callback(|_| SimulationEditorMsg::EditSeed);
        let onsubmit = ctx.link().callback(|_| SimulationEditorMsg::Save);
        let oncancel = ctx.link().callback(|_| SimulationEditorMsg::Cancel);

        // TODO: Explain the module requirements
        let class = classes!(
            "grid",
            "grid-cols-1",
            "lg:grid-cols-2",
            "gap-2",
            "lg:grid-rows-2",
            props.class.clone()
        );

        html! {
            <form action="javascript:void(0);" {class}>
                <div class="max-w-prose h-full flex-col align-start">
                    {docs()}

                    if let Some(err) = &self.error {
                        <div class="alert"><pre>{err.to_string()}</pre></div>
                    }

                    <div class="flex flex-row justify-around">
                        <button type="button" onclick={oncancel}>{"Cancel"}</button>
                        <button type="button" onclick={onsubmit}>{"Submit"}</button>
                    </div>
                </div>

                <div class="flex flex-col h-full lg:row-start-2 space-y-2">
                    <p>{"Initial state"}</p>
                    <div class="space-x-4">
                        <label for="seed">{"Seed"}</label>
                        <input name="seed" ref={self.seed_ref.clone()} type="text" {oninput} />
                    </div>
                    <div class="flex flex-grow justify-start items-start h-full min-h-0">
                        <div class="box-square">
                            <Grid prev={Blocks::default()} next={self.preview} class="h-full w-full" />
                        </div>
                    </div>
                </div>

                <div class="flex flex-col lg:row-start-1 lg:col-start-2 lg:row-span-2">
                    <label for="source">{"Module"}</label>
                    <textarea name="source" ref={self.source_ref.clone()} />
                </div>
            </form>
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
            SimulationEditorMsg::EditSeed => {
                let link = ctx.link().clone();
                self.pending_update = Some(Timeout::new(500, move || {
                    link.send_message(SimulationEditorMsg::UpdatePreview)
                }));

                false
            }

            SimulationEditorMsg::UpdatePreview => {
                self.pending_update = None;
                self.preview = Blocks::from_seed(self.current_seed().unwrap_or_default());
                true
            }

            SimulationEditorMsg::Save => {
                let seed = self.current_seed().unwrap_or_default();

                match self.current_module() {
                    Ok(module) => {
                        let value = SimulationEditorValue { seed, module };
                        props.onsubmit.emit(Some(value));
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

impl SimulationEditor {
    fn current_seed(&self) -> eyre::Result<u64> {
        let seed = self.seed_ref.cast::<HtmlInputElement>();
        let seed = seed.ok_or_eyre("no input element")?;
        Ok(seed.value().parse()?)
    }

    fn current_module(&self) -> eyre::Result<Module> {
        let source = self.source_ref.cast::<HtmlTextAreaElement>();
        let source = source.ok_or_eyre("no textarea element")?;
        Ok(Module::new(source.value())?)
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

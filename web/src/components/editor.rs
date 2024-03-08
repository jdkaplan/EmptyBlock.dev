use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct EditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub seed: u64,
    pub source: String,

    pub onsubmit: Callback<Option<EditorValue>>,
}

#[derive(PartialEq, Debug)]
pub struct EditorValue {
    pub seed: u64,
    pub source: String,
}

pub struct Editor {
    seed_ref: NodeRef,
    source_ref: NodeRef,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EditorMsg {
    Save,
    Cancel,
}

impl Component for Editor {
    type Message = EditorMsg;

    type Properties = EditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            seed_ref: NodeRef::default(),
            source_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let seed = props.seed.to_string();
        let source = props.source.clone();
        let rows = 1 + source.lines().count();

        let onsubmit = ctx.link().callback(|_| EditorMsg::Save);
        let oncancel = ctx.link().callback(|_| EditorMsg::Cancel);

        // TODO: Explain the module requirements

        html! {
            <form action="javascript:void(0);" class={props.class.clone()}>
                <label for="seed">{"Seed"}</label>
                <input name="seed" ref={self.seed_ref.clone()} type="text" value={seed} />

                <label for="source">{"Module"}</label>
                <textarea name="source" ref={self.source_ref.clone()} value={source} rows={rows.to_string()} />
                <div class="flex flex-row justify-around">
                    <button type="button" onclick={oncancel}>{"Cancel"}</button>
                    <button type="button" onclick={onsubmit}>{"Submit"}</button>
                </div>
            </form>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            EditorMsg::Save => {
                let seed = self.seed_ref.cast::<HtmlInputElement>().unwrap();
                let source = self.source_ref.cast::<HtmlTextAreaElement>().unwrap();

                props.onsubmit.emit(Some(EditorValue {
                    seed: seed.value().parse().unwrap_or_default(),
                    source: source.value(),
                }));

                false
            }
            EditorMsg::Cancel => {
                props.onsubmit.emit(None);
                false
            }
        }
    }
}

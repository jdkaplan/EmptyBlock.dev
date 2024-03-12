use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::apps::trellis::{Config, Layout, Secrets};

#[derive(Properties, PartialEq, Debug)]
pub struct BoardEditorProps {
    #[prop_or_default]
    pub class: Classes,

    pub config: Config,

    pub onsubmit: Callback<Option<Config>>,
}

pub struct BoardEditor {
    secrets_ref: NodeRef,
    layout_ref: NodeRef,

    errors: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BoardEditorMsg {
    Save,
    Cancel,
}

impl Component for BoardEditor {
    type Message = BoardEditorMsg;

    type Properties = BoardEditorProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            secrets_ref: NodeRef::default(),
            layout_ref: NodeRef::default(),
            errors: Vec::new(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let onsubmit = ctx.link().callback(|_| BoardEditorMsg::Save);
        let oncancel = ctx.link().callback(|_| BoardEditorMsg::Cancel);

        html! {
            <div class={classes!("flex", "flex-col", props.class.clone())}>
                <div class="max-w-prose info self-center">{docs()}</div>

                if !self.errors.is_empty() {
                    <div class="alert">
                    { for self.errors.iter().map(|err| html! { <pre>{err}</pre> }) }
                    </div>
                }

                <form action="javascript:void(0);" class="flex flex-col">
                    <label for="secrets">{"Secrets"}</label>
                    <textarea name="secrets" ref={self.secrets_ref.clone()} />

                    <label for="layout">{"Module"}</label>
                    <textarea name="layout" ref={self.layout_ref.clone()} />

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

            let secrets = self.secrets_ref.cast::<HtmlTextAreaElement>().unwrap();
            let layout = self.layout_ref.cast::<HtmlTextAreaElement>().unwrap();

            fill_textarea(&secrets, &props.config.secrets).expect("always valid JSON");
            fill_textarea(&layout, &props.config.layout).expect("always valid JSON");
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        match msg {
            BoardEditorMsg::Save => {
                let secrets = self.secrets_ref.cast::<HtmlInputElement>().unwrap();
                let layout = self.layout_ref.cast::<HtmlTextAreaElement>().unwrap();

                let mut errors = Vec::new();
                let mut config = Config::default();

                match serde_json::from_str(&secrets.value()) {
                    Ok(secrets) => config.secrets = secrets,
                    Err(err) => errors.push(err.to_string()),
                }

                match serde_json::from_str(&layout.value()) {
                    Ok(layout) => config.layout = layout,
                    Err(err) => errors.push(err.to_string()),
                }

                if errors.is_empty() {
                    props.onsubmit.emit(Some(config));
                    false
                } else {
                    self.errors = errors;
                    true
                }
            }
            BoardEditorMsg::Cancel => {
                props.onsubmit.emit(None);
                false
            }
        }
    }
}

fn docs() -> Html {
    html! { <>
    <p>{"This could definitely use a better editing UI, but this is still much better than opening the console to write local storage directly."}</p>
    </> }
}

fn fill_textarea<T>(textarea: &HtmlTextAreaElement, value: &T) -> serde_json::Result<()>
where
    T: ?Sized + serde::Serialize,
{
    let json = serde_json::to_string_pretty(value)?;

    let rows = 1 + json.lines().count() as u32;

    textarea.set_value(&json);
    textarea.set_rows(rows);
    Ok(())
}

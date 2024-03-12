use gloo::timers::callback::Timeout;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct NoteProps {
    #[prop_or_default]
    pub initial: AttrValue,

    #[prop_or_default]
    pub onchange: Callback<String>,
}

pub struct Note {
    props: NoteProps,
    form_ref: NodeRef,
    textarea_ref: NodeRef,
    pending_save: Option<Timeout>,
}

pub enum NoteMsg {
    Edited,
    Saved,
}

impl Component for Note {
    type Message = NoteMsg;

    type Properties = NoteProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            props: ctx.props().clone(),
            textarea_ref: NodeRef::default(),
            form_ref: NodeRef::default(),
            pending_save: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            NoteMsg::Edited => {
                // TODO: Prevent navigation while waiting for save
                let timeout = {
                    let link = ctx.link().clone();
                    Timeout::new(1000, move || link.send_message(NoteMsg::Saved))
                };

                self.pending_save = Some(timeout);
                true
            }
            NoteMsg::Saved => {
                self.pending_save = None;

                let textarea = self.textarea_ref.cast::<HtmlTextAreaElement>().unwrap();
                let text = textarea.value();

                self.props.onchange.emit(text);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut classes = classes!("w-full", "h-full", "resize-none", "border");

        if self.pending_save.is_some() {
            classes.extend(classes!(
                "border-yellow-500",
                "focus:border-yellow-500",
                "focus:ring-yellow-500",
            ));
        }

        let oninput = {
            let link = ctx.link().clone();
            move |_| link.send_message(NoteMsg::Edited)
        };

        html! {
            <form class="w-full h-full p-2" ref={self.form_ref.clone()}>
                <label for="note_text" class="sr-only">{"Note text"}</label>
                <textarea
                    id="note_text"
                    class={classes}
                    ref={self.textarea_ref.clone()}
                    oninput={oninput}
                    ~defaultValue={self.props.initial.clone()}
                >
                </textarea>
            </form>
        }
    }
}

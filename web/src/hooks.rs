use std::borrow::Cow;

use gloo::utils::{body, document};
use web_sys::js_sys::Array;
use web_sys::wasm_bindgen::JsValue;
use yew::prelude::{hook, use_effect};

#[hook]
pub fn use_title(title: impl Into<Cow<'static, str>>) {
    let title = title.into();
    use_effect(move || {
        let previous = document().title();
        document().set_title(&title);

        move || {
            document().set_title(&previous);
        }
    });
}

#[hook]
pub fn use_body_class(classes: Vec<&str>) {
    let extra = classes
        .into_iter()
        .map(JsValue::from_str)
        .collect::<Array>();

    use_effect(move || {
        let previous = body().class_list().value();
        body()
            .class_list()
            .add(&extra)
            .expect("class names validated before call");

        move || {
            body().class_list().set_value(&previous);
        }
    });
}

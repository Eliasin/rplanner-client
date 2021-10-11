use std::rc::Rc;
use web_sys::console;
use yew::{function_component, html, use_ref, use_state, Callback};

mod delta;
mod quill_wrapper;

#[function_component(HelloWorld)]
fn hello_world() -> Html {
    let quill_wrapper = use_ref(|| Option::<quill_wrapper::QuillWrapper>::None);
    let quill_content = use_state(|| String::new());

    let check_is_init = Rc::new({
        let quill_wrapper = quill_wrapper.clone();
        move || {
            let mut is_init = false;
            if quill_wrapper.borrow().is_some() {
                is_init = true;
            }

            is_init
        }
    });

    let spawn_onclick = {
        let check_is_init = check_is_init.clone();
        let quill_wrapper = quill_wrapper.clone();
        Callback::from(move |_| {
            if !check_is_init() {
                let new_quill = quill_wrapper::QuillWrapper::new();
                new_quill.spawn_quill(String::from("#quill"));

                quill_wrapper.replace(Some(new_quill));
            }
        })
    };

    let content_onclick = {
        let check_is_init = check_is_init.clone();
        let quill_wrapper = quill_wrapper.clone();
        let quill_content = quill_content.clone();
        Callback::from(move |_| {
            if check_is_init() {
                let content = quill_wrapper
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .get_content()
                    .clone();

                quill_content.set(content);
            }
        })
    };

    let quill_content: &str = quill_content.as_ref();
    let quill_markdown = match delta::parse_delta_to_markdown(quill_content) {
        Ok(v) => v,
        Err(e) => e.to_string(),
    };

    console::log_1(&format!("{}", quill_markdown.clone()).into());
    console::log_1(&format!("{}", quill_content.clone()).into());
    html! { <>
              <button onclick={spawn_onclick} >{"Spawn Quill"}</button>
               <button onclick={content_onclick} >{"Check Contents"}</button>
              <div>{quill_markdown}</div>
              <div id="quill-container">
                <div id="quill"></div>
              </div>
            </>
    }
}

fn main() {
    yew::start_app::<HelloWorld>();
}

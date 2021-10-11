use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/quill_wrapper.js")]
extern "C" {
    pub type QuillWrapper;

    #[wasm_bindgen(constructor)]
    pub fn new() -> QuillWrapper;

    #[wasm_bindgen(method)]
    pub fn spawn_quill(this: &QuillWrapper, selector: String);

    #[wasm_bindgen(method)]
    pub fn get_content(this: &QuillWrapper) -> String;

    #[wasm_bindgen(method)]
    pub fn get_content_from_index(this: &QuillWrapper) -> String;

    #[wasm_bindgen(method)]
    pub fn get_content_from_index_and_length(
        this: &QuillWrapper,
        index: u32,
        length: u32,
    ) -> String;
}

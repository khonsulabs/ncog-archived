use wasm_bindgen::prelude::*;
mod webapp;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<webapp::App>();

    Ok(())
}

#![recursion_limit = "8192"]
use wasm_bindgen::prelude::*;
mod strings;
mod webapp;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<webapp::App>();

    Ok(())
}

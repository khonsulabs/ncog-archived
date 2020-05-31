#![recursion_limit = "8192"]
use wasm_bindgen::prelude::*;
mod api;
mod login;
mod strings;
mod webapp;

#[macro_use]
extern crate log;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<webapp::App>();

    Ok(())
}

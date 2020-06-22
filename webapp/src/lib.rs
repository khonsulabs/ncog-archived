#![recursion_limit = "8192"]
use wasm_bindgen::prelude::*;

#[macro_use]
mod internal_macros {
    #[allow(dead_code)]
    #[macro_export]
    macro_rules! todo {
        () => { error!("not yet implemented {}:{}", file!(), line!()) };
        ($($arg:tt)+) => { error!( "not yet implemented {}:{}: {}", file!(), line!(), std::format_args!($($arg)+))};
    }

    #[macro_export]
    macro_rules! hash_map {
        ($($key:expr => $value:expr),+) => {{
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )+
            map
        }};
    }

    #[macro_export]
    macro_rules! hash_set {
        ($($value:expr),+) => {{
            let mut set = std::collections::HashSet::new();
            $(
                set.insert($value);
            )+
            set
        }};
    }
}

#[macro_use]
mod webapp;

#[macro_use]
extern crate log;

#[cfg(debug_assertions)]
const MAX_LOG_LEVEL: log::Level = log::Level::Trace;
#[cfg(not(debug_assertions))]
const MAX_LOG_LEVEL: log::Level = log::Level::Info;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(MAX_LOG_LEVEL));
    yew::start_app::<webapp::App>();

    Ok(())
}

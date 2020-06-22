pub use include_dir;
pub mod alert;
pub mod flash;
pub mod forms;
pub mod localization;
pub mod markdown;
pub mod modal;
pub mod static_page;
pub mod title;
pub mod validations;

use chrono::{naive::NaiveDateTime, DateTime, Utc};

pub fn wasm_utc_now() -> DateTime<Utc> {
    let timestamp = js_sys::Date::new_0().get_time();
    let secs = timestamp.floor();
    let nanoes = (timestamp - secs) * 1_000_000_000f64;
    let naivetime = NaiveDateTime::from_timestamp(secs as i64, nanoes as u32);
    DateTime::from_utc(naivetime, Utc)
}

pub mod prelude {
    pub use crate::{
        alert::Alert,
        flash::{self, Flash},
        forms::prelude::*,
        modal::Modal,
        title::Title,
        validations::prelude::*,
    };
    pub use chrono::{naive::NaiveDateTime, DateTime, Utc};
    pub use std::rc::Rc;
}

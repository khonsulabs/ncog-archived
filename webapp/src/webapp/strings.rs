use lazy_static::lazy_static;
use shared::{
    fluent_templates::loader::Loader,
    localization::{LOCALES, US_ENGLISH},
};
use yew::prelude::*;

use khonsuweb::markdown::render_markdown;

pub fn localize(name: &str) -> Html {
    let source = LOCALES.lookup(&US_ENGLISH, name);
    render_markdown(&source)
}

pub fn localize_raw(name: &str) -> Html {
    let source = LOCALES.lookup(&US_ENGLISH, name);
    source.into()
}

pub mod prelude {
    pub use super::localize;
    pub use khonsuweb::localization::StringBundle;
}

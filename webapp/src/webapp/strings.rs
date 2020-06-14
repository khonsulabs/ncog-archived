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

pub fn localize_raw(name: &str) -> String {
    let source = LOCALES.lookup(&US_ENGLISH, name);
    source
}

pub mod prelude {
    pub use super::localize;
    pub use khonsuweb::localization::StringBundle;
}

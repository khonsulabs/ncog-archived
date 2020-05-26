use lazy_static::lazy_static;
use yew::prelude::*;

use khonsuweb::{include_dir::include_dir, localization::StringBundle};
lazy_static! {
    static ref STATIC_BUNDLE: StringBundle = { StringBundle::load(&include_dir!("strings")) };
}

pub fn localize(name: &str) -> Html {
    STATIC_BUNDLE.localize(name)
}

pub mod prelude {
    pub use super::localize;
    pub use khonsuweb::localization::StringBundle;
}

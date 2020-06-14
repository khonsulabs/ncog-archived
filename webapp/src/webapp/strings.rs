use shared::{
    fluent_templates::{fluent_bundle::FluentValue, loader::Loader},
    localization::{LOCALES, US_ENGLISH},
};
use std::collections::HashMap;
use yew::prelude::*;

use khonsuweb::markdown::render_markdown;

pub fn localize(name: &str) -> Html {
    render_markdown(&localize_raw(name))
}

pub fn localize_with_args(name: &str, args: &HashMap<String, FluentValue>) -> Html {
    render_markdown(&localize_raw_with_args(name, args))
}

pub fn localize_raw(name: &str) -> String {
    LOCALES.lookup(&US_ENGLISH, name)
}

pub fn localize_raw_with_args(name: &str, args: &HashMap<String, FluentValue>) -> String {
    LOCALES.lookup_with_args(&US_ENGLISH, name, args)
}

pub trait Namable {
    fn name(&self) -> &'static str;
}

pub trait LocalizableName {
    fn localized_name(&self) -> String;
}

#[macro_export]
macro_rules! localize_html {
    ($name:expr) => {
        crate::webapp::strings::localize($name)
    };
    ($name:expr, $($key:expr => $value:expr),+) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(String::from($key), shared::fluent_templates::fluent_bundle::FluentValue::from($value));
        )+
        crate::webapp::strings::localize_with_args($name, &args)
    }};
}
#[macro_export]
macro_rules! localize {
    ($name:expr) => {
        crate::webapp::strings::localize_raw($name)
    };
    ($name:expr, $($key:expr => $value:expr),+) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(String::from($key), shared::fluent_templates::fluent_bundle::FluentValue::from($value));
        )+
        crate::webapp::strings::localize_raw_with_args($name, &args)
    }};
}

impl<T> LocalizableName for T
where
    T: Namable,
{
    fn localized_name(&self) -> String {
        localize!(&self.name())
    }
}

pub mod prelude {
    pub use super::localize;
    pub use khonsuweb::localization::StringBundle;
}

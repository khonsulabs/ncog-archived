use include_dir::include_dir;
use unic_langid::{langid, LanguageIdentifier};

pub const US_ENGLISH: LanguageIdentifier = langid!("en-US");

fluent_templates::static_loader! {
    // Declare our `StaticLoader` named `LOCALES`.
    pub static LOCALES = {
        // The directory of localisations and fluent resources.
        locales: "./shared/src/strings",
        // The language to falback on if something is not present.
        fallback_language: "en-US",
    };
}

fn unused() {
    include_dir!("./src/strings");
}

use super::markdown::render_markdown;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use yew::prelude::*;

#[derive(Clone)]
pub struct StringBundle {
    markdown: Arc<RwLock<HashMap<String, String>>>,
}

use include_dir::{Dir, DirEntry};

impl StringBundle {
    pub fn load(dir: &Dir) -> StringBundle {
        let mut markdown = HashMap::new();
        for entry in dir.find("**/*.md").unwrap() {
            if let DirEntry::File(file) = entry {
                let path = entry.path().to_str().unwrap().to_owned();
                let source = String::from_utf8(file.contents().to_vec())
                    .unwrap_or_else(|_| panic!("Invalid UTF-8 in {}", path));
                markdown.insert(path.replace("\\", "/"), source);
            }
        }
        StringBundle {
            markdown: Arc::new(RwLock::new(markdown)),
        }
    }
    pub fn localize(&self, name: &str) -> Html {
        let markdown = self.markdown.read().unwrap();
        match markdown.get(name) {
            Some(html) => render_markdown(html.as_ref()),
            None => panic!("Unknown string {}", name),
        }
    }
}

pub mod prelude {
    pub use super::StringBundle;
}

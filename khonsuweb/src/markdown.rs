/// Original author of this code is [Nathan Ringo](https://github.com/remexre)
/// Source: https://github.com/acmumn/mentoring/blob/master/web-client/src/view/markdown.rs
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag};
use yew::virtual_dom::{Classes, VNode, VTag, VText};
use yew::{html, Html};

/// Renders a string of Markdown to HTML with the default options (footnotes
/// disabled, tables enabled).
pub fn render_markdown(src: &str) -> Html {
    let mut elems = vec![];
    let mut spine = vec![];

    macro_rules! add_child {
        ($child:expr) => {{
            let l = spine.len();
            assert_ne!(l, 0);
            spine[l - 1].add_child($child);
        }};
    }

    for ev in Parser::new_ext(src, Options::empty()) {
        match ev {
            Event::Start(tag) => {
                spine.push(make_tag(tag));
            }
            Event::End(tag) => {
                let l = spine.len();
                assert!(l >= 1);
                let mut top = spine.pop().unwrap();
                if let Tag::CodeBlock(_) = tag {
                    let mut pre = VTag::new("pre");
                    pre.add_child(top.into());
                    top = pre;
                } else if let Tag::Table(aligns) = tag {
                    for r in top.children.iter_mut() {
                        if let VNode::VTag(ref mut vtag) = r {
                            for (i, c) in vtag.children.iter_mut().enumerate() {
                                if let VNode::VTag(ref mut vtag) = c {
                                    let class = match aligns[i] {
                                        Alignment::None => None,
                                        Alignment::Left => Some("text-left"),
                                        Alignment::Center => Some("text-center"),
                                        Alignment::Right => Some("text-right"),
                                    };
                                    if let Some(class) = class {
                                        let mut classes = Classes::new();
                                        classes.push(class);
                                        vtag.add_attribute("class", &classes);
                                    }
                                }
                            }
                        }
                    }
                } else if let Tag::TableHead = tag {
                    for c in top.children.iter_mut() {
                        if let VNode::VTag(ref mut vtag) = c {
                            vtag.add_attribute("scope", &"col");
                        }
                    }
                }
                if l == 1 {
                    elems.push(top);
                } else {
                    spine[l - 2].add_child(top.into());
                }
            }
            Event::Text(text) => add_child!(VText::new(text.to_string()).into()),
            Event::SoftBreak => add_child!(VText::new("\n".to_string()).into()),
            Event::HardBreak => add_child!(VTag::new("br").into()),
            Event::Html(html) => add_child!(VText::new(html.to_string()).into()),
            _ => panic!("Unknown event: {:#?}", ev),
        }
    }

    if elems.len() == 1 {
        VNode::VTag(Box::new(elems.pop().unwrap()))
    } else {
        html! {
            <div>{ for elems.into_iter() }</div>
        }
    }
}

fn add_class(vtag: &mut VTag, class: &str) {
    let mut classes: Classes = vtag
        .attributes
        .get("class")
        .map(AsRef::as_ref)
        .unwrap_or("")
        .into();
    classes.push(class);
    vtag.add_attribute("class", &classes);
}

fn make_tag(t: Tag) -> VTag {
    match t {
        Tag::Paragraph => VTag::new("p"),
        //Tag::Rule => VTag::new("hr"),
        Tag::Heading(n) => {
            assert!(n > 0);
            assert!(n < 7);
            VTag::new(format!("h{}", n))
        }
        Tag::BlockQuote => {
            let mut el = VTag::new("blockquote");
            add_class(&mut el, "blockquote");
            el
        }
        Tag::CodeBlock(kind) => {
            let mut el = VTag::new("code");
            if let CodeBlockKind::Fenced(lang) = kind {
                // Different color schemes may be used for different code blocks,
                // but a different library (likely js based at the moment) would be necessary to actually provide the
                // highlighting support by locating the language classes and applying dom transforms
                // on their contents.
                match lang.as_ref() {
                    "html" => add_class(&mut el, "html-language"),
                    "rust" => add_class(&mut el, "rust-language"),
                    "java" => add_class(&mut el, "java-language"),
                    "c" => add_class(&mut el, "c-language"),
                    _ => {} // Add your own language highlighting support
                };
            }
            el
        }
        Tag::List(None) => VTag::new("ul"),
        Tag::List(Some(1)) => VTag::new("ol"),
        Tag::List(Some(ref start)) => {
            let mut el = VTag::new("ol");
            el.add_attribute("start", start);
            el
        }
        Tag::Item => VTag::new("li"),
        Tag::Table(_) => {
            let mut el = VTag::new("table");
            add_class(&mut el, "table");
            el
        }
        Tag::TableHead => VTag::new("th"),
        Tag::TableRow => VTag::new("tr"),
        Tag::TableCell => VTag::new("td"),
        Tag::Emphasis => {
            let mut el = VTag::new("span");
            add_class(&mut el, "is-italic");
            el
        }
        Tag::Strong => {
            let mut el = VTag::new("span");
            add_class(&mut el, "has-text-weight-bold");
            el
        }
        Tag::Link(_type, ref href, ref title) => {
            let mut el = VTag::new("a");
            el.add_attribute("href", href);
            if title.as_ref() != "" {
                el.add_attribute("title", title);
            }
            el
        }
        Tag::Image(_type, ref src, ref title) => {
            let mut el = VTag::new("img");
            el.add_attribute("src", src);
            if title.as_ref() != "" {
                el.add_attribute("title", title);
            }
            el
        }
        Tag::Strikethrough => VTag::new("strike"),
        Tag::FootnoteDefinition(ref _footnote_id) => VTag::new("span"), // Footnotes are not rendered as anything special
    }
}

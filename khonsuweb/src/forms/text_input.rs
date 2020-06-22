use super::storage::FormStorage;
use std::{collections::HashMap, rc::Rc, str::FromStr};
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct TextInput<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Clone + FromStr + ToString + std::fmt::Debug + PartialEq + 'static,
{
    props: Props<T, V>,
    text_value: String,
    input: NodeRef,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Clone + FromStr + ToString + std::fmt::Debug + PartialEq + 'static,
{
    #[prop_or_default]
    pub on_value_changed: Callback<Option<V>>,
    pub storage: FormStorage<Option<V>>,
    pub field: T,
    pub errors: Option<Rc<HashMap<T, Vec<Rc<Html>>>>>,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub readonly: bool,
}

pub enum Message {
    KeyPressed,
}

impl<T, V> Component for TextInput<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Clone + FromStr + ToString + std::fmt::Debug + PartialEq + 'static,
{
    type Message = Message;
    type Properties = Props<T, V>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let text_value = props
            .storage
            .value()
            .unwrap_or(None)
            .map(|v| v.to_string())
            .unwrap_or_default();
        TextInput {
            props,
            link,
            input: NodeRef::default(),
            text_value,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::KeyPressed => {
                if let Some(input) = self.input.cast::<HtmlInputElement>() {
                    self.text_value = input.value();
                    if self.text_value.is_empty() {
                        self.props.storage.update_with_invalid_hint(None, false);
                        self.props.storage.update_invalid_hint(false);
                        self.props.on_value_changed.emit(None);
                    } else if let Ok(value) = V::from_str(&self.text_value) {
                        self.props
                            .storage
                            .update_with_invalid_hint(Some(value.clone()), false);
                        self.props.storage.update_invalid_hint(false);
                        self.props.on_value_changed.emit(Some(value));
                    } else {
                        self.props.storage.update_invalid_hint(true);
                        self.props
                            .on_value_changed
                            .emit(self.props.storage.value().unwrap_or_default());
                    }
                }
            }
        }
        false
    }

    fn view(&self) -> Html {
        let errors = self
            .props
            .errors
            .as_ref()
            .map(|errors| errors.get(&self.props.field).cloned());
        let css_class = match &errors {
            Some(errors) => match errors {
                Some(_) => "input is-danger",
                None => "input",
            },
            None => "input",
        };
        html! {
            <div class="control">
                <input class=css_class ref=self.input.clone() type="text" value=self.text_value placeholder=&self.props.placeholder onchange=self.link.callback(|_| Message::KeyPressed) oninput=self.link.callback(|_| Message::KeyPressed) disabled=self.props.disabled readonly=self.props.readonly />
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.text_value = props
            .storage
            .unchecked_value()
            .map(|v| v.to_string())
            .unwrap_or_default();
        self.props = props;
        true
    }
}

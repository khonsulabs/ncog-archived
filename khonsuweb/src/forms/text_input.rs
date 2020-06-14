use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct TextInput<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    props: Props<T>,
    input: NodeRef,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    #[prop_or_default]
    pub on_value_changed: Callback<String>,
    pub storage: Rc<RefCell<String>>,
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

impl<T> Component for TextInput<T>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        TextInput {
            props,
            link,
            input: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::KeyPressed => {
                if let Some(input) = self.input.cast::<HtmlInputElement>() {
                    *self.props.storage.borrow_mut() = input.value();
                    self.props.on_value_changed.emit(input.value());
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
            .map(|errors| errors.get(&self.props.field).map(|errors| errors.clone()));
        let css_class = match &errors {
            Some(errors) => match errors {
                Some(_) => "input is-danger",
                None => "input",
            },
            None => "input",
        };
        html! {
            <div class="control">
                <input class=css_class ref=self.input.clone() type="text" value=self.props.storage.borrow() placeholder=&self.props.placeholder onchange=self.link.callback(|_| Message::KeyPressed) oninput=self.link.callback(|_| Message::KeyPressed) disabled=self.props.disabled readonly=self.props.readonly />
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

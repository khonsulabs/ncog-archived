use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Radio<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Copy + Eq + 'static,
{
    props: Props<T, V>,
    input: NodeRef,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Copy + Eq + 'static,
{
    #[prop_or_default]
    pub on_value_changed: Callback<V>,
    pub storage: Rc<RefCell<V>>,
    pub field: T,
    pub errors: Option<Rc<HashMap<T, Vec<Rc<Html>>>>>,
    pub options: Vec<(String, V)>,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
}

pub enum Message<V>
where
    V: Copy + Eq + 'static,
{
    RadioSelected(V),
}

impl<T, V> Component for Radio<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Copy + Eq + 'static,
{
    type Message = Message<V>;
    type Properties = Props<T, V>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            input: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::RadioSelected(value) => {
                *self.props.storage.borrow_mut() = value;
                self.props.on_value_changed.emit(value);
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
                { self.props.options.iter().map(|(label, value)| self.render_option(label, *value)).collect::<Html>() }
            </div>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl<T, V> Radio<T, V>
where
    T: Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static,
    V: Copy + Eq + 'static,
{
    fn render_option(&self, label: &str, value: V) -> Html {
        html! {
            <label class="radio">
                <input type="radio" onclick=self.link.callback(move |_| Message::RadioSelected(value)) checked=*self.props.storage.borrow() == value disabled=self.props.disabled />
                { label }
            </label>
        }
        //  <input class=css_class ref=self.input.clone() type="text" value=self.props.storage.borrow() placeholder=&self.props.placeholder onchange=self.link.callback(|_| Message::KeyPressed) oninput=self.link.callback(|_| Message::KeyPressed) disabled=self.props.disabled readonly=self.props.readonly />
    }
}

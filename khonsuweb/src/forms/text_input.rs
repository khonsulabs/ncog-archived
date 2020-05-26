use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct TextInput {
    props: Props,
    input: NodeRef,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub on_value_changed: Callback<String>,
    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
}

pub enum Message {
    KeyPressed,
}

impl Component for TextInput {
    type Message = Message;
    type Properties = Props;

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
                    self.props.value = input.value();
                    self.props.on_value_changed.emit(input.value());
                }
            }
        }
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="control">
                <input ref=self.input.clone() class="input" type="text" value=self.props.value placeholder=&self.props.placeholder onchange=self.link.callback(|_| Message::KeyPressed) oninput=self.link.callback(|_| Message::KeyPressed) disabled=self.props.disabled />
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

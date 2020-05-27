use yew::prelude::*;

pub struct Button {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub action: Callback<MouseEvent>,
    pub label: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub processing: bool,
    #[prop_or_default]
    pub css_class: String,
}

pub enum Message {
    Action(MouseEvent),
}

impl Component for Button {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::Action(e) => {
                self.props.action.emit(e);
            }
        }
        false
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let content = if self.props.processing {
            html! {
                <img src="/loader.svg" width=20 height="30" />
            }
        } else {
            html! { { &self.props.label } }
        };
        html! {
            <div class="control">
                <button class=format!("button {}", self.props.css_class) disabled=self.props.disabled onclick=self.link.callback(|e: MouseEvent| Message::Action(e))>
                    { content }
                </button>
            </div>
        }
    }
}

use yew::prelude::*;

pub struct Modal {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub visible: bool,

    #[prop_or_default]
    pub close_requested: Callback<()>,
    #[prop_or_default]
    pub close_button: bool,
    #[prop_or_default]
    pub children: Children,
}

pub enum Message {
    CloseRequested,
}

impl Component for Modal {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::CloseRequested => {
                self.props.close_requested.emit(());
            }
        }
        false
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class=self.css_class()>
                <div class="modal-background" onclick=self.link.callback(|e: MouseEvent| {e.prevent_default(); Message::CloseRequested}) />
                <div class="modal-content">
                    { self.props.children.clone() }
                </div>
                { self.close_button() }
            </div>
        }
    }
}

impl Modal {
    fn css_class(&self) -> &'static str {
        if self.props.visible {
            "modal is-active"
        } else {
            "modal"
        }
    }

    fn close_button(&self) -> Html {
        if self.props.close_button {
            html! {
                <button class="modal-close is-large" aria-label="close" onclick=self.link.callback(|e: MouseEvent| {e.prevent_default(); Message::CloseRequested}) />
            }
        } else {
            Html::default()
        }
    }
}

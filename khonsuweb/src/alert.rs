use crate::{forms::button::Button, modal::Modal, title::Title};
use yew::prelude::*;

pub struct Alert {
    props: Props,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub visible: bool,
    pub title: String,
    pub message: String,
    pub primary_button_label: String,
    #[prop_or("is-danger".to_owned())]
    pub primary_button_class: String,
    pub primary_button_action: Callback<MouseEvent>,
    #[prop_or_default]
    pub cancel_button_label: String,
    #[prop_or_default]
    pub cancel_button_action: Callback<MouseEvent>,
    #[prop_or_default]
    pub cancel_button_class: String,
}

impl Component for Alert {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <Modal visible=self.props.visible>
                <div class="box">
                    <Title>{ &self.props.title }</Title>
                    <p>{ &self.props.message }</p>
                    { self.buttons() }
                </div>
            </Modal>
        }
    }
}

impl Alert {
    fn buttons(&self) -> Html {
        let cancel_button = if self.props.cancel_button_label.is_empty() {
            Html::default()
        } else {
            html! {
                <Button
                    label=&self.props.cancel_button_label
                    css_class=&self.props.cancel_button_class
                    action=self.props.cancel_button_action.clone()
                />
            }
        };

        html! {
            <div class="level">
                <div class="level-left">
                    <div class="level-item">
                        {cancel_button}
                    </div>
                </div>
                <div class="level-right">
                    <div class="level-item">
                        <Button
                            label=&self.props.primary_button_label
                            css_class=&self.props.primary_button_class
                            action=self.props.primary_button_action.clone()
                        />
                    </div>
                </div>
            </div>
        }
    }
}

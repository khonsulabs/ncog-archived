use yew::prelude::*;

pub struct Label {
    props: Props,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub text: String,
}

impl Component for Label {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props = props;
        true
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <label class="label">{ &self.props.text }</label>
        }
    }
}

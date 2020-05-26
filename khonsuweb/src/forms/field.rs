use super::label::Label;
use yew::prelude::*;

pub struct Field {
    props: Props,
}

#[derive(Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub label: String,
    #[prop_or_default]
    pub help: String,
    #[prop_or_default]
    pub children: Children,
}

impl Component for Field {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let label = if self.props.label.len() > 0 {
            html! {<Label text=self.props.label.clone()/>}
        } else {
            html! {}
        };
        let help = if self.props.help.len() > 0 {
            html! {<p class="help">{ &self.props.help }</p>}
        } else {
            html! {}
        };
        html! {
            <div class="field">
                { label }
                { self.props.children.render() }
                { help }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

use yew::prelude::*;

pub struct StaticPage {
    props: Props,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub title: String,
    pub content: Html,
    pub set_title: Callback<String>,
}

impl Component for StaticPage {
    type Properties = Props;
    type Message = ();

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let component = Self { props };
        component.update_title();
        component
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.update_title();
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns is-centered">
                <div class="column is-half">
                    { self.props.content.clone() }
                </div>
            </div>
        }
    }
}

impl StaticPage {
    fn update_title(&self) {
        self.props.set_title.emit(self.props.title.clone());
    }
}

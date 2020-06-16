use super::{AppRoute, LoggedInUser};
use shared::permissions::Claim;
use std::sync::Arc;
use yew::prelude::*;
use yew_router::prelude::*;

pub mod edit_form;
pub mod entity_list;
pub mod roles;
pub mod users;

pub struct Dashboard {
    props: Props,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub set_title: Callback<String>,
}

pub enum Message {}

impl Component for Dashboard {
    type Message = Message;
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
        require_permission!(&self.props.user, read_claim());

        html!(
            <div class="columns is-centered">
                <div class="column is-half">

                </div>
            </div>
        )
    }
}

pub fn read_claim() -> Claim {
    Claim::new("backoffice", Some("dashboard"), None, "read")
}

fn render_heading_with_add_button(
    title: &'static str,
    add_route: AppRoute,
    add_caption: &'static str,
) -> Html {
    html! {
        <div class="level">
            <div class="level-left">
                <h2>{localize!(title)}</h2>
            </div>
            <div class="level-right">
                <RouterButton<AppRoute> route=add_route classes="button is-success" >
                    <strong>{ localize!(add_caption) }</strong>
                </RouterButton<AppRoute>>
            </div>
        </div>
    }
}

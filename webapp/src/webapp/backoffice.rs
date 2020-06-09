use super::LoggedInUser;
use crate::require_permission;
use shared::permissions::Claim;
use std::sync::Arc;
use yew::prelude::*;

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

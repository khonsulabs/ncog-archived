use super::{has_permission, invalid_permissions, LoggedInUser};
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
        info!("Current permssions: {:#?}", self.props.user);
        if !has_permission(
            &self.props.user,
            Claim::new("backoffice", Some("dashboard"), None, "read"),
        ) {
            return invalid_permissions();
        }

        html!(
            <div class="columns is-centered">
                <div class="column is-half">

                </div>
            </div>
        )
    }
}

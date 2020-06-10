use crate::{
    require_permission,
    webapp::{
        api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
        has_permission, LoggedInUser,
    },
};
use shared::{
    iam::{IAMRequest, IAMResponse, User},
    permissions::Claim,
    ServerResponse,
};
use std::sync::Arc;
use yew::prelude::*;
use yew_router::{
    agent::{RouteAgentBridge, RouteRequest},
    route::Route,
};

pub struct EditUser {
    props: Props,
}
#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub editing_id: Option<i64>,
    pub set_title: Callback<String>,
}

impl Component for EditUser {
    type Message = ();
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
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
        require_permission!(&self.props.user, read_claim(self.props.editing_id));

        let editable = has_permission(&self.props.user, write_claim(self.props.editing_id));
        html! {
            // TODO THIS IS WHERE YOU LEFT OFF JON -- Start trying to use the form controls.
            <div>
                // <form>
                //     <TextInput value=self.pending_message.clone() on_value_changed=self.link.callback(|value| Message::MessageInputChanged(value)) placeholder="Type your message here..." />
                //     <Button label="Send" disabled=disable_button css_class="is-success" action=self.link.callback(|e: ClickEvent| {e.prevent_default(); Message::SendMessage})/>
                // </form>
            </div>
        }
    }
}

pub fn read_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("users"), id, "read")
}

pub fn write_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("users"), id, "write")
}

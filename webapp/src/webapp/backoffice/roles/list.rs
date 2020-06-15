use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    backoffice::{entity_list::EntityList, roles::summary_list::default_action_buttons},
    LoggedInUser,
};
use shared::{
    iam::{roles_list_claim, IAMRequest, IAMResponse, RoleSummary},
    ServerResponse,
};
use std::sync::Arc;
use yew::prelude::*;

pub struct RolesList {
    api: ApiBridge,
    props: Props,
    roles: Option<Vec<RoleSummary>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub set_title: Callback<String>,
}

pub enum Message {
    WsMessage(AgentResponse),
}

impl Component for RolesList {
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        Self {
            props,
            api,
            roles: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::WsMessage(agent_response) => match agent_response {
                AgentResponse::Response(ws_response) => match ws_response.result {
                    ServerResponse::IAM(iam_response) => match iam_response {
                        IAMResponse::RolesList(users) => {
                            self.roles = Some(users);
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                },
                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.initialize();
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, roles_list_claim());
        html!(
            <div class="container">
                <h2>{localize_html!("list-roles")}</h2>
                <EntityList<RoleSummary> entities=self.roles.clone() action_buttons=default_action_buttons()/>
            </div>
        )
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.initialize();
        }

        self.props.set_title.emit(localize!("list-roles"));
    }
}

impl RolesList {
    fn initialize(&mut self) {
        self.api
            .send(AgentMessage::Request(shared::ServerRequest::IAM(
                IAMRequest::RolesList,
            )))
    }
}

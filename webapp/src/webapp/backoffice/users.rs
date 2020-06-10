use super::{
    super::api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    LoggedInUser,
};
use crate::require_permission;
use shared::{
    iam::{IAMRequest, IAMResponse, User},
    permissions::Claim,
    ServerResponse,
};
use std::sync::Arc;
use yew::prelude::*;
use yew_router::prelude::*;

pub struct Users {
    api: ApiBridge,
    props: Props,
    users: Option<Vec<User>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub set_title: Callback<String>,
}

pub enum Message {
    WsMessage(AgentResponse),
}

impl Component for Users {
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        Self {
            props,
            api,
            users: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::WsMessage(agent_response) => match agent_response {
                AgentResponse::Response(ws_response) => match ws_response.result {
                    ServerResponse::Authenticated { .. } => {
                        self.initialize();
                        false
                    }
                    ServerResponse::IAM(iam_response) => match iam_response {
                        IAMResponse::UsersList(users) => {
                            self.users = Some(users);
                            true
                        }
                    },
                    _ => false,
                },
                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, read_claim());

        match self.users.clone() {
            Some(users) => html!(
                <div class="container">
                    <table class="table">
                        <thead>
                            <tr>
                                <td>{"Id"}</td>
                                <td>{"Screen Name"}</td>
                                <td>{"Created At"}</td>
                            </tr>
                        </thead>
                        <tbody>
                            { users.iter().map(|u| self.render_user(u)).collect::<Html>() }
                        </tbody>
                    </table>
                </div>
            ),
            None => {
                html! {
                    <div>
                        <h1>{"Loading..."}</h1>
                        <progress class="progress is-primary" max="100"/>
                    </div>
                }
            }
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.api.send(AgentMessage::RegisterBroadcastHandler);
            self.initialize();
        }
    }
}

pub fn read_claim() -> Claim {
    Claim::new("iam", Some("users"), None, "list")
}

impl Users {
    fn initialize(&mut self) {
        self.api
            .send(AgentMessage::Request(shared::ServerRequest::IAM(
                IAMRequest::UsersList,
            )))
    }

    fn render_user(&self, user: &User) -> Html {
        html! {
            <tr>
                <td>{ user.id }</td>
                <td>{ user.screenname.as_ref().unwrap_or(&"<Not Chosen>".to_owned())}</td>
                <td>{ user.created_at }</td>
            </tr>
        }
    }
}

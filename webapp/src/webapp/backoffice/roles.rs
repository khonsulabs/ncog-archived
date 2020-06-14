use crate::{
    localize, localize_html, require_permission,
    webapp::{
        api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
        backoffice::entity_list::{EntityList, ListableEntity},
        strings::{LocalizableName, Namable},
        AppRoute, LoggedInUser,
    },
};
use shared::{
    iam::{roles_list_claim, roles_read_claim, IAMRequest, IAMResponse, RoleSummary},
    permissions::Claim,
    ServerResponse,
};
use std::sync::Arc;
use yew::prelude::*;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum RoleFields {
    Id,
    Name,
    CreatedAt,
}

impl Namable for RoleFields {
    fn name(&self) -> &'static str {
        match self {
            Self::Id => "role-fields-id",
            Self::Name => "role-fields-name",
            Self::CreatedAt => "role-fields-created-at",
        }
    }
}

impl ListableEntity for RoleSummary {
    type Entity = RoleSummary;

    fn table_head() -> Html {
        html! {
            <tr>
                <td>{ RoleFields::Id.localized_name() }</td>
                <td>{ RoleFields::Name.localized_name() }</td>
                <td></td>
            </tr>
        }
    }

    fn render_entity(role: &Self::Entity) -> Html {
        html! {
            <tr>
                <td>{ role.id }</td>
                <td>{ &role.name }</td>
                <td>
                    // <RouterButton<AppRoute> route=AppRoute::BackOfficeRoleEdit(role.id) classes="button is-primary" >
                    //     <strong>{ localize("edit") }</strong>
                    // </RouterButton<AppRoute>>
                </td>
            </tr>
        }
    }
}

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
                    ServerResponse::Authenticated { .. } => {
                        self.initialize();
                        false
                    }
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
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, roles_list_claim());
        html!(
            <div class="container">
                <h2>{localize_html!("list-roles")}</h2>
                <EntityList<RoleSummary> entities=self.roles.clone()/>
            </div>
        )
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.api.send(AgentMessage::RegisterBroadcastHandler);
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

use crate::{
    require_permission,
    webapp::{
        api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
        backoffice::{
            entity_list::{EntityList, ListableEntity},
            users::fields::UserFields,
        },
        strings::{localize, localize_raw, LocalizableName},
        AppRoute, LoggedInUser,
    },
};
use shared::{
    iam::{IAMRequest, IAMResponse, User},
    permissions::Claim,
    ServerResponse,
};
use std::{rc::Rc, sync::Arc};
use yew::prelude::*;
use yew_router::prelude::*;

pub struct UsersList {
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

impl Component for UsersList {
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
                    ServerResponse::IAM(iam_response) => match iam_response {
                        IAMResponse::UsersList(users) => {
                            self.users = Some(users);
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
        require_permission!(&self.props.user, read_claim());
        html!(
            <div class="container">
                <h2>{localize("list-users")}</h2>
                <EntityList<Self> entities=self.users.clone() action_buttons=default_action_buttons() />
            </div>
        )
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.initialize();
        }

        self.props.set_title.emit(localize_raw("list-users"));
    }
}

pub fn read_claim() -> Claim {
    Claim::new("iam", Some("users"), None, "list")
}

impl UsersList {
    fn initialize(&mut self) {
        self.api
            .send(AgentMessage::Request(shared::ServerRequest::IAM(
                IAMRequest::UsersList,
            )))
    }
}

impl ListableEntity for UsersList {
    type Entity = User;

    fn table_head() -> Html {
        html! {
            <tr>
                <td>{ UserFields::Id.localized_name() }</td>
                <td>{ UserFields::Screenname.localized_name() }</td>
                <td>{ UserFields::CreatedAt.localized_name() }</td>
                <td></td>
            </tr>
        }
    }

    fn render_entity(
        user: &Self::Entity,
        action_buttons: Option<Rc<Box<dyn Fn(&Self::Entity) -> Html>>>,
    ) -> Html {
        html! {
            <tr>
                <td>{ user.id }</td>
                <td>{ user.screenname.as_ref().unwrap_or(&localize_raw("not-set"))}</td>
                <td>{ user.created_at }</td>
                <td>
                    { Self::render_action_buttons(action_buttons, user) }
                </td>
            </tr>
        }
    }
}

pub fn default_action_buttons() -> Option<Rc<Box<dyn Fn(&User) -> Html>>> {
    Some(Rc::new(Box::new(render_default_action_buttons)))
}

fn render_default_action_buttons(user: &User) -> Html {
    html! {
        <RouterButton<AppRoute> route=AppRoute::BackOfficeUserEdit(user.id) classes="button is-primary" >
            <strong>{ localize("edit") }</strong>
        </RouterButton<AppRoute>>
    }
}

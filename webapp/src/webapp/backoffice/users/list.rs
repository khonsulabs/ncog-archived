use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    backoffice::{
        entity_list::{body::EntityRenderer, EntityList},
        users::fields::UserFields,
    },
    strings::{localize, localize_raw, LocalizableName},
    AppRoute, EditingId, LoggedInUser,
};
use khonsuweb::title::Title;
use shared::{
    iam::{users_list_claim, IAMRequest, IAMResponse, User},
    ServerResponse,
};
use std::{rc::Rc, sync::Arc};
use yew::prelude::*;
use yew_router::prelude::*;

pub struct UsersList {
    api: ApiBridge,
    props: Props,
    users: Option<Rc<Vec<User>>>,
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
        let callback = link.callback(Message::WsMessage);
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
                            self.users = Some(Rc::new(users));
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
        require_permission!(&self.props.user, users_list_claim());
        html!(
            <div class="container">
                <Title>{localize("list-users")}</Title>

                { standard(self.users.clone()) }
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

impl UsersList {
    fn initialize(&mut self) {
        self.api
            .send(AgentMessage::Request(shared::ServerRequest::IAM(
                IAMRequest::UsersList,
            )))
    }
}

pub fn standard_head() -> Html {
    html! {
        <tr>
            <td>{ UserFields::Id.localized_name() }</td>
            <td>{ UserFields::Screenname.localized_name() }</td>
            <td>{ UserFields::CreatedAt.localized_name() }</td>
            <td></td>
        </tr>
    }
}

pub fn standard_row() -> EntityRenderer<User> {
    EntityRenderer::new(|user: &User| {
        html! {
            <tr>
                <td>{ user.id.unwrap() }</td>
                <td>{ user.screenname.as_ref().unwrap_or(&localize_raw("not-set"))}</td>
                <td>{ user.created_at }</td>
                <td>
                    <RouterButton<AppRoute> route=AppRoute::BackOfficeUserEdit(EditingId::Id(user.id.unwrap())) classes="button is-primary" >
                        <strong>{ localize("edit") }</strong>
                    </RouterButton<AppRoute>>
                </td>
            </tr>
        }
    })
}

pub fn standard(entities: Option<Rc<Vec<User>>>) -> Html {
    html! {
        <EntityList<User>
            header=standard_head()
            row=standard_row()
            entities=entities
            />
    }
}

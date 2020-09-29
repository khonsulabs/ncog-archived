use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    backoffice::{entity_list::EntityList, render_heading_with_add_button, roles::summary_list},
    has_permission, AppRoute, EditingId, LoggedInUser,
};
use khonsuweb::prelude::*;
use shared::{
    iam::{
        roles_create_claim, roles_delete_claim, roles_list_claim, IAMRequest, IAMResponse,
        RoleSummary,
    },
    ServerRequest, ServerResponse,
};
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};
use yew::prelude::*;
use yew_router::prelude::*;

pub struct RolesList {
    api: ApiBridge,
    props: Props,
    roles: Option<Rc<RwLock<Vec<RoleSummary>>>>,
    link: ComponentLink<Self>,
    pending_delete_id: Option<i64>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub set_title: Callback<String>,
}

pub enum RolesListMessage {
    WsMessage(AgentResponse),
    RoleRequestDelete(i64),
    RoleDelete,
    RoleCancelDelete,
}

impl Component for RolesList {
    type Message = RolesListMessage;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(RolesListMessage::WsMessage);
        let api = ApiAgent::bridge(callback);
        Self {
            props,
            api,
            link,
            roles: None,
            pending_delete_id: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            RolesListMessage::WsMessage(agent_response) => match agent_response {
                AgentResponse::Response(ws_response) => match ws_response {
                    ServerResponse::IAM(iam_response) => match iam_response {
                        IAMResponse::RolesList(users) => {
                            self.roles = Some(Rc::new(RwLock::new(users)));
                            true
                        }
                        IAMResponse::RoleDeleted(id) => {
                            if let Some(roles) = &self.roles {
                                let mut roles = roles.write().expect("Error locking roles");
                                roles.retain(|role| role.id.unwrap() != id);
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    },
                    _ => false,
                },
                _ => false,
            },
            RolesListMessage::RoleRequestDelete(id) => {
                self.pending_delete_id = Some(id);
                true
            }
            RolesListMessage::RoleCancelDelete => {
                self.pending_delete_id = None;
                true
            }
            RolesListMessage::RoleDelete => {
                if let Some(id) = self.pending_delete_id {
                    self.api.send(AgentMessage::Request(ServerRequest::IAM(
                        IAMRequest::RoleDelete(id),
                    )));
                }
                self.pending_delete_id = None;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.initialize();
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, roles_list_claim());
        let can_create = has_permission(&self.props.user, roles_create_claim());
        let link = self.link.clone();
        let user = self.props.user.clone();
        html!(
            <div>
                <Alert
                    visible=self.pending_delete_id.is_some()
                    title=localize!("delete-role")
                    message=localize!("delete-irreversable")
                    primary_button_action=self.link.callback(|e: MouseEvent| {e.prevent_default(); RolesListMessage::RoleDelete})
                    primary_button_label=localize!("delete")
                    cancel_button_action=self.link.callback(|e: MouseEvent| {e.prevent_default(); RolesListMessage::RoleCancelDelete})
                    cancel_button_label=localize!("cancel")
                    />
                <section class="section content">
                    { render_heading_with_add_button("list-roles", AppRoute::BackOfficeRoleEdit(EditingId::New), "add-role", !can_create) }

                    <EntityList<RoleSummary>
                        header=summary_list::standard_head()
                        row=summary_list::row(move |role| {
                            let id = role.id.unwrap();
                            let can_delete = has_permission(&user, roles_delete_claim(role.id));
                            html! {
                                // TODO create a grouped control to make this easier
                                <div class="field is-grouped">
                                    <p class="control">
                                        <RouterButton<AppRoute> route=AppRoute::BackOfficeRoleEdit(EditingId::Id(role.id.unwrap())) classes="button is-primary" >
                                            <strong>{ localize!("edit") }</strong>
                                        </RouterButton<AppRoute>>
                                    </p>
                                    <p class="control">
                                        <Button
                                            label=localize!("delete")
                                            css_class="is-danger"
                                            disabled=!can_delete
                                            action=link.callback(move |_| RolesListMessage::RoleRequestDelete(id))
                                        />
                                    </p>
                                </div>
                            }
                        })
                        entities=self.roles.clone()
                    />
                </section>
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

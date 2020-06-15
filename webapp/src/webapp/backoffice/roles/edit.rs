use crate::{
    localize, localize_html, require_permission,
    webapp::{
        api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
        backoffice::roles::RoleFields,
        has_permission,
        strings::LocalizableName,
        LoggedInUser,
    },
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use shared::{
    iam::{roles_read_claim, roles_update_claim, IAMRequest, IAMResponse, RoleSummary},
    ServerRequest, ServerResponse,
};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use yew::prelude::*;

pub struct EditRole {
    api: ApiBridge,
    props: Props,
    role: Role,
    link: ComponentLink<Self>,
    flash_message: Option<flash::Message>,
    is_saving: bool,
}

#[derive(Debug)]
pub struct Role {
    id: Rc<RefCell<String>>,
    name: Rc<RefCell<String>>,
}

pub enum Message {
    ValueChanged,
    Save,
    WsMessage(AgentResponse),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub editing_id: Option<i64>,
    pub set_title: Callback<String>,
}

impl Component for EditRole {
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let role = Role {
            id: Rc::new(RefCell::new(String::new())),
            name: Rc::new(RefCell::new(String::new())),
        };
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        Self {
            props,
            role,
            link,
            api,
            is_saving: false,
            flash_message: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::ValueChanged => true,
            Message::Save => {
                let role = RoleSummary {
                    id: self.props.editing_id,
                    name: self.role.name.borrow().clone(),
                };
                self.api.send(AgentMessage::Request(ServerRequest::IAM(
                    IAMRequest::RoleSave(role),
                )));

                self.is_saving = true;
                true
            }
            Message::WsMessage(agent_response) => match agent_response {
                AgentResponse::Response(ws_response) => match ws_response.result {
                    ServerResponse::IAM(response) => match response {
                        IAMResponse::Role(role) => {
                            if let Some(id) = &self.props.editing_id {
                                if id == &role.id.unwrap() {
                                    *self.role.id.borrow_mut() = id.to_string();
                                    *self.role.name.borrow_mut() = role.name.clone();
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        IAMResponse::RoleSaved(_) => {
                            self.flash_message = Some(flash::Message::new(
                                flash::Kind::Success,
                                localize!("saved-role"),
                                Duration::from_secs(5),
                            ));
                            self.is_saving = false;
                            true
                        }
                        _ => false,
                    },
                    ServerResponse::Error { message } => {
                        if let Some(message) = message {
                            self.flash_message = Some(flash::Message::new(
                                flash::Kind::Danger,
                                message,
                                Duration::from_secs(5),
                            ));
                        }
                        self.is_saving = false;
                        true
                    }
                    _ => false,
                },
                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        if self.props.editing_id.is_some() {
            self.initialize()
        }
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, roles_read_claim(self.props.editing_id));

        let errors = self.validate().map(|errors| {
            errors.translate(|e| match e.error {
                ValidationError::NotPresent => {
                    localize_html!("form-field-required", "field" => e.primary_field().localized_name())
                }
            })
        });

        let readonly = self.is_saving
            || !has_permission(&self.props.user, roles_update_claim(self.props.editing_id));
        let can_save = !readonly && !errors.is_some();
        html! {
            <div>
                <h2>{localize_html!("edit-role")}</h2>
                <form>
                    <flash::Flash message=self.flash_message.clone() />
                    <Field<RoleFields> field=RoleFields::Id errors=errors.clone()>
                        <Label text=RoleFields::Id.localized_name() />
                        <TextInput<RoleFields> field=RoleFields::Id storage=self.role.id.clone() readonly=true errors=errors.clone() />
                    </Field<RoleFields>>
                    <Field<RoleFields> field=RoleFields::Name errors=errors.clone()>
                        <Label text=RoleFields::Name.localized_name() />
                        <TextInput<RoleFields> field=RoleFields::Name storage=self.role.name.clone() readonly=readonly on_value_changed=self.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                    </Field<RoleFields>>
                    <Button
                        label=localize!("save-role")
                        disabled=!can_save
                        css_class="is-primary"
                        action=self.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                        processing=self.is_saving
                    />
                </form>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.initialize();
        }
        self.props.set_title.emit(localize!("edit-role"))
    }
}

impl EditRole {
    fn validate(&self) -> Option<Rc<ErrorSet<RoleFields>>> {
        ModelValidator::new()
            .with_field(RoleFields::Name, self.role.name.is_present())
            .validate()
    }

    fn initialize(&mut self) {
        if let Some(role_id) = self.props.editing_id {
            self.api
                .send(AgentMessage::Request(shared::ServerRequest::IAM(
                    IAMRequest::RoleGet(role_id),
                )))
        }
    }
}

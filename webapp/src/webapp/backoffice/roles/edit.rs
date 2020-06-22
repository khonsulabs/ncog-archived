use crate::webapp::{
    api::{AgentMessage, ApiBridge},
    backoffice::{
        edit_form::{EditForm, ErrorMap, Form, Handled, Message, Props},
        entity_list::EntityList,
        render_heading_with_add_button,
        roles::fields::RoleFields,
        roles::permission_statements::{self},
    },
    strings::Namable,
    AppRoute, EditingId,
};
use khonsuweb::prelude::*;
use shared::{
    iam::{
        roles_create_claim, roles_read_claim, roles_update_claim, IAMRequest, IAMResponse,
        PermissionStatement, RoleSummary,
    },
    permissions::Claim,
    ServerRequest, ServerResponse,
};
use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Default)]
pub struct Role {
    id: FormStorage<Option<i64>>,
    name: FormStorage<Option<String>>,
    permission_statements: Option<Rc<RwLock<Vec<PermissionStatement>>>>,
    pending_permission_deletion: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum RoleMessage {
    PermissionRequestDelete(i64),
    PermissionDelete,
    PermissionCancelDelete,
}

impl Form for Role {
    type Message = RoleMessage;
    type Fields = RoleFields;
    fn title(is_new: bool) -> &'static str {
        if is_new {
            "add-role"
        } else {
            "edit-role"
        }
    }

    fn route_for(id: EditingId, _owning_id: Option<i64>) -> AppRoute {
        AppRoute::BackOfficeRoleEdit(id)
    }

    fn load_request(&self, props: &Props) -> Option<ServerRequest> {
        props
            .editing_id
            .existing_id()
            .map(|role_id| ServerRequest::IAM(IAMRequest::RoleGet(role_id)))
    }

    fn save(&mut self, props: &Props, api: &mut ApiBridge) {
        let role = RoleSummary {
            id: props.editing_id.existing_id(),
            name: self.name.value().unwrap_or(None).unwrap_or_default(),
        };

        api.send(AgentMessage::Request(ServerRequest::IAM(
            IAMRequest::RoleSave(role),
        )));
    }

    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled {
        match response {
            ServerResponse::IAM(response) => match response {
                IAMResponse::Role(role) => {
                    if let Some(id) = &role.id {
                        self.id.update(Some(*id));
                        self.name.update(Some(role.name));
                        self.permission_statements =
                            Some(Rc::new(RwLock::new(role.permission_statements)));
                        Handled::ShouldRender(true)
                    } else {
                        Handled::ShouldRender(false)
                    }
                }
                IAMResponse::RoleSaved(new_id) => Handled::Saved {
                    label: "saved-role",
                    new_id,
                },
                IAMResponse::PermissionStatementDeleted(id) => {
                    if let Some(permission_statements) = &self.permission_statements {
                        let mut permission_statements = permission_statements.write().unwrap();
                        permission_statements.retain(|stmt| stmt.id.unwrap() != id);
                    }
                    Handled::ShouldRender(true)
                }
                _ => Handled::ShouldRender(false),
            },
            _ => unreachable!("Unexpected message from server"),
        }
    }

    fn render(
        &self,
        edit_form: &EditForm<Self>,
        readonly: bool,
        can_save: bool,
        errors: Option<Rc<ErrorMap<Self::Fields>>>,
    ) -> Html {
        let is_new = edit_form.props.editing_id.is_new();
        let id = match edit_form.props.editing_id {
            EditingId::Id(_) => {
                html! {
                    <Field<RoleFields> field=RoleFields::Id errors=errors.clone()>
                        <Label text=RoleFields::Id.localized_name() />
                        <TextInput<RoleFields, i64> field=RoleFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                    </Field<RoleFields>>
                }
            }
            EditingId::New => Html::default(),
        };

        let permission_statements = if is_new {
            Html::default()
        } else {
            let link = edit_form.link.clone();
            html! {
                <section class="section content">
                    { render_heading_with_add_button(
                        RoleFields::PermissionStatements.name(),
                        AppRoute::BackOfficeRolePermissionStatementEdit(edit_form.props.editing_id.existing_id().expect("Editing a permission without an role is not allowed"),
                            EditingId::New
                        ),
                        "add-permission-statement",
                        readonly) }

                    <EntityList<PermissionStatement>
                        header=permission_statements::list::standard_head()
                        row=permission_statements::list::row(move |permission_statement| {
                            let id = permission_statement.id.unwrap();
                            html! {
                                // TODO create a grouped control to make this easier
                                <div class="field is-grouped">
                                    <p class="control">
                                        <RouterButton<AppRoute> route=AppRoute::BackOfficeRolePermissionStatementEdit(permission_statement.role_id.unwrap(), EditingId::Id(permission_statement.id.unwrap())) classes="button is-primary" >
                                            <strong>{ localize!("edit") }</strong>
                                        </RouterButton<AppRoute>>
                                    </p>
                                    <p class="control">
                                        <Button
                                            label=localize!("delete")
                                            css_class="is-danger"
                                            disabled=readonly
                                            action=link.callback(move |_| Message::FormMessage(RoleMessage::PermissionRequestDelete(id)))
                                        />
                                    </p>
                                </div>
                            }
                        })
                        entities=self.permission_statements.clone()
                        />
                </section>
            }
        };

        html! {
            <div>
                <Alert
                    visible=self.pending_permission_deletion.is_some()
                    title=localize!("delete-permission-statement")
                    message=localize!("delete-irreversable")
                    primary_button_action=edit_form.link.callback(|e: MouseEvent| {e.prevent_default(); Message::FormMessage(RoleMessage::PermissionDelete)})
                    primary_button_label=localize!("delete")
                    cancel_button_action=edit_form.link.callback(|e: MouseEvent| {e.prevent_default(); Message::FormMessage(RoleMessage::PermissionCancelDelete)})
                    cancel_button_label=localize!("cancel")
                    />
                <section class="section content">
                    <Title>{localize!(Self::title(is_new))}</Title>
                    <form>
                        <flash::Flash message=edit_form.flash_message.clone() />
                        { id }
                        <Field<RoleFields> field=RoleFields::Name errors=errors.clone()>
                            <Label text=RoleFields::Name.localized_name() />
                            <TextInput<RoleFields, String> field=RoleFields::Name storage=self.name.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                        </Field<RoleFields>>
                        <Button
                            label=localize!("save-role")
                            disabled=!can_save
                            css_class="is-primary"
                            action=edit_form.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                            processing=edit_form.is_saving
                        />
                    </form>
                </section>

                { permission_statements }
            </div>
        }
    }

    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>> {
        ModelValidator::default()
            .with_field(RoleFields::Name, self.name.is_present())
            .validate()
    }

    fn read_claim(id: Option<i64>) -> Claim {
        roles_read_claim(id)
    }
    fn update_claim(id: Option<i64>) -> Claim {
        roles_update_claim(id)
    }
    fn create_claim() -> Claim {
        roles_create_claim()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _props: &Props,
        api: &mut ApiBridge,
    ) -> ShouldRender {
        match message {
            RoleMessage::PermissionRequestDelete(id) => {
                self.pending_permission_deletion = Some(id);
            }
            RoleMessage::PermissionDelete => {
                api.send(AgentMessage::Request(ServerRequest::IAM(
                    IAMRequest::PermissionStatemenetDelete(
                        self.pending_permission_deletion.unwrap(),
                    ),
                )));
                self.pending_permission_deletion = None;
            }
            RoleMessage::PermissionCancelDelete => {
                self.pending_permission_deletion = None;
            }
        }
        true
    }
}

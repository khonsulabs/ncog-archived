use crate::webapp::{
    api::{AgentMessage, ApiBridge},
    backoffice::{
        edit_form::{EditForm, Form, Handled, Message, Props},
        entity_list::EntityList,
        roles::fields::RoleFields,
        roles::permission_statements,
    },
    strings::LocalizableName,
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use shared::{
    iam::{
        roles_create_claim, roles_read_claim, roles_update_claim, IAMRequest, IAMResponse,
        PermissionStatement, RoleSummary,
    },
    permissions::Claim,
    ServerRequest, ServerResponse,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct Role {
    id: Rc<RefCell<String>>,
    name: Rc<RefCell<String>>,
    permission_statements: Vec<PermissionStatement>,
}

impl Form for Role {
    type Fields = RoleFields;
    fn title() -> &'static str {
        "edit-role"
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
            name: self.name.borrow().clone(),
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
                        *self.id.borrow_mut() = id.to_string();
                        *self.name.borrow_mut() = role.name;
                        self.permission_statements = role.permission_statements;
                        Handled::ShouldRender(true)
                    } else {
                        Handled::ShouldRender(false)
                    }
                }
                IAMResponse::RoleSaved(_) => Handled::Saved("saved-role"),
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
        errors: Option<Rc<HashMap<Self::Fields, Vec<Rc<Html>>>>>,
    ) -> Html {
        html! {
            <div>
                <h2>{localize_html!("edit-role")}</h2>
                <form>
                    <flash::Flash message=edit_form.flash_message.clone() />
                    <Field<RoleFields> field=RoleFields::Id errors=errors.clone()>
                        <Label text=RoleFields::Id.localized_name() />
                        <TextInput<RoleFields> field=RoleFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                    </Field<RoleFields>>
                    <Field<RoleFields> field=RoleFields::Name errors=errors.clone()>
                        <Label text=RoleFields::Name.localized_name() />
                        <TextInput<RoleFields> field=RoleFields::Name storage=self.name.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                    </Field<RoleFields>>
                    <Button
                        label=localize!("save-role")
                        disabled=!can_save
                        css_class="is-primary"
                        action=edit_form.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                        processing=edit_form.is_saving
                    />
                </form>

                <h2>{RoleFields::PermissionStatements.localized_name()}</h2>
                <EntityList<PermissionStatement> entities=self.permission_statements.clone() action_buttons=permission_statements::list::default_action_buttons() />
            </div>
        }
    }

    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>> {
        ModelValidator::new()
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
}

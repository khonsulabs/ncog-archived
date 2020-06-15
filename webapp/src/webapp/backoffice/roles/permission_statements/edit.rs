use crate::webapp::{
    api::{AgentMessage, ApiBridge},
    backoffice::{
        edit_form::{EditForm, Form, Handled, Message, Props},
        entity_list::EntityList,
        roles::permission_statements::fields::PermissionStatementFields,
    },
    strings::LocalizableName,
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use shared::{
    iam::{
        roles_read_claim, roles_update_claim, IAMRequest, IAMResponse, PermissionStatement,
        RoleSummary,
    },
    permissions::Claim,
    ServerRequest, ServerResponse,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct PermissionStatementForm {
    id: Rc<RefCell<String>>,
    service: Rc<RefCell<String>>,
    resource_type: Rc<RefCell<String>>,
    resource_id: Rc<RefCell<String>>,
    action: Rc<RefCell<String>>,
    allow: Rc<RefCell<bool>>,
}

impl Form for PermissionStatementForm {
    type Fields = PermissionStatementFields;
    fn title() -> &'static str {
        "edit-permission-statement"
    }
    fn load_request(&self, props: &Props) -> Option<ServerRequest> {
        props
            .editing_id
            .map(|id| ServerRequest::IAM(IAMRequest::PermissionStatementGet(id)))
    }

    fn save(&mut self, props: &Props, api: &mut ApiBridge) {
        todo!("Need to save permission statements")
    }

    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled {
        match response {
            ServerResponse::IAM(response) => match response {
                IAMResponse::PermissionStatement(statement) => {
                    if let Some(id) = &statement.id {
                        *self.id.borrow_mut() = id.to_string();
                        *self.service.borrow_mut() = statement.service.unwrap_or_default();
                        *self.resource_type.borrow_mut() =
                            statement.resource_type.unwrap_or_default();
                        *self.resource_id.borrow_mut() = statement
                            .resource_id
                            .map(|id| id.to_string())
                            .unwrap_or_default();
                        *self.action.borrow_mut() = statement.action.unwrap_or_default();
                        *self.allow.borrow_mut() = statement.allow;
                        Handled::ShouldRender(true)
                    } else {
                        Handled::ShouldRender(false)
                    }
                }
                // TODO IAMResponse::RoleSaved(_) => Handled::Saved("saved-permission-statement"),
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
                <h2>{localize_html!("edit-permission-statement")}</h2>
                <form>
                    <flash::Flash message=edit_form.flash_message.clone() />
                    <Field<PermissionStatementFields> field=PermissionStatementFields::Id errors=errors.clone()>
                        <Label text=PermissionStatementFields::Id.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::Service errors=errors.clone()>
                        <Label text=PermissionStatementFields::Service.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::Service storage=self.service.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-service") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::ResourceType errors=errors.clone()>
                        <Label text=PermissionStatementFields::ResourceType.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::ResourceType storage=self.resource_type.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-resource-type") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::ResourceId errors=errors.clone()>
                        <Label text=PermissionStatementFields::ResourceId.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::ResourceId storage=self.resource_id.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-resource-id") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::Action errors=errors.clone()>
                        <Label text=PermissionStatementFields::Action.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::Action storage=self.action.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-action") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Radio<PermissionStatementFields, bool>
                        field=PermissionStatementFields::Allow
                        errors=errors.clone()
                        storage=self.allow.clone()
                        disabled=readonly
                        on_value_changed=edit_form.link.callback(|_| Message::ValueChanged)
                        options=vec![(localize!("action-denied"), false), (localize!("action-allowed"), true)]
                    />

                    <Button
                        label=localize!("save-permission-statement")
                        disabled=!can_save
                        css_class="is-primary"
                        action=edit_form.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                        processing=edit_form.is_saving
                    />
                </form>
            </div>
        }
    }

    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>> {
        ModelValidator::new().validate()
    }

    fn read_claim(id: Option<i64>) -> Claim {
        // TODO this isn't right.
        roles_read_claim(id)
    }
    fn update_claim(id: Option<i64>) -> Claim {
        // TODO this isn't right.
        roles_update_claim(id)
    }
}

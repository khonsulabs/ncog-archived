use crate::webapp::{
    api::{AgentMessage, ApiBridge},
    backoffice::{
        edit_form::{EditForm, Form, Handled, Message, Props},
        roles::permission_statements::fields::PermissionStatementFields,
    },
    strings::LocalizableName,
    EditingId,
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use shared::{
    iam::{
        roles_create_claim, roles_read_claim, roles_update_claim, IAMRequest, IAMResponse,
        PermissionStatement,
    },
    permissions::Claim,
    ServerRequest, ServerResponse,
};
use std::{collections::HashMap, rc::Rc};
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct PermissionStatementForm {
    id: FormStorage<String>,
    service: FormStorage<String>,
    resource_type: FormStorage<String>,
    resource_id: FormStorage<String>,
    action: FormStorage<String>,
    allow: FormStorage<bool>,
    comment: FormStorage<String>,
}

impl Form for PermissionStatementForm {
    type Fields = PermissionStatementFields;
    fn title(is_new: bool) -> &'static str {
        match is_new {
            true => "add-permission-statement",
            false => "edit-permission-statement",
        }
    }

    fn route_for(id: EditingId, owning_id: Option<i64>) -> String {
        format!(
            "/backoffice/roles/{}/permissions/{}",
            owning_id.expect("Need to have an owning_id"),
            id
        )
    }

    fn load_request(&self, props: &Props) -> Option<ServerRequest> {
        props
            .editing_id
            .existing_id()
            .map(|id| ServerRequest::IAM(IAMRequest::PermissionStatementGet(id)))
    }

    fn save(&mut self, props: &Props, api: &mut ApiBridge) {
        let statement = PermissionStatement {
            id: props.editing_id.existing_id(),
            role_id: props.owning_id,
            service: self.service.value_as_option(),
            resource_type: self.resource_type.value_as_option(),
            resource_id: self.resource_id.value_as_option().map(|id| {
                id.parse()
                    .expect("Need to catch invalid ints in validation")
            }),
            action: self.action.value_as_option(),
            allow: self.allow.value(),
            comment: self.comment.value_as_option(),
        };

        info!("{:#?}", statement);

        api.send(AgentMessage::Request(ServerRequest::IAM(
            IAMRequest::PermissionStatementSave(statement),
        )));
    }

    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled {
        match response {
            ServerResponse::IAM(response) => match response {
                IAMResponse::PermissionStatement(statement) => {
                    if let Some(id) = &statement.id {
                        self.id.update(id.to_string());

                        self.service.update(statement.service.unwrap_or_default());
                        self.resource_type
                            .update(statement.resource_type.unwrap_or_default());
                        self.resource_id.update(
                            statement
                                .resource_id
                                .map(|id| id.to_string())
                                .unwrap_or_default(),
                        );
                        self.action.update(statement.action.unwrap_or_default());
                        self.allow.update(statement.allow);
                        Handled::ShouldRender(true)
                    } else {
                        Handled::ShouldRender(false)
                    }
                }
                IAMResponse::PermissionStatementSaved(new_id) => Handled::Saved {
                    label: "saved-permission-statement",
                    new_id,
                },
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
        let is_new = edit_form.props.editing_id.is_new();
        let id = match edit_form.props.editing_id {
            EditingId::Id(_) => {
                html! {
                    <Field<PermissionStatementFields> field=PermissionStatementFields::Id errors=errors.clone()>
                        <Label text=PermissionStatementFields::Id.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                    </Field<PermissionStatementFields>>
                }
            }
            EditingId::New => Html::default(),
        };

        html! {
            <div>
                <Title>{localize!(Self::title(is_new))}</Title>
                <form>
                    <flash::Flash message=edit_form.flash_message.clone() />

                    { id }

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

                    <Field<PermissionStatementFields> field=PermissionStatementFields::Comment errors=errors.clone()>
                        <Label text=PermissionStatementFields::Comment.localized_name() />
                        <TextInput<PermissionStatementFields> field=PermissionStatementFields::Comment storage=self.comment.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("permission-statement-comment-placeholder") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

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
    fn create_claim() -> Claim {
        // TODO this isn't right.
        roles_create_claim()
    }
}

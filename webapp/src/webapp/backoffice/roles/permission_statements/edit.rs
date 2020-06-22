use crate::webapp::{
    api::{AgentMessage, ApiBridge},
    backoffice::{
        edit_form::{EditForm, ErrorMap, Form, Handled, Message, Props},
        roles::permission_statements::fields::PermissionStatementFields,
    },
    strings::Namable,
    AppRoute, EditingId,
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
use std::rc::Rc;
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct PermissionStatementForm {
    id: FormStorage<Option<i64>>,
    service: FormStorage<Option<String>>,
    resource_type: FormStorage<Option<String>>,
    resource_id: FormStorage<Option<i64>>,
    action: FormStorage<Option<String>>,
    allow: FormStorage<bool>,
    comment: FormStorage<Option<String>>,
}

impl Form for PermissionStatementForm {
    type Message = ();
    type Fields = PermissionStatementFields;
    fn title(is_new: bool) -> &'static str {
        if is_new {
            "add-permission-statement"
        } else {
            "edit-permission-statement"
        }
    }

    fn route_for(id: EditingId, owning_id: Option<i64>) -> AppRoute {
        AppRoute::BackOfficeRolePermissionStatementEdit(owning_id.unwrap(), id)
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
            service: self.service.value().unwrap_or_default(),
            resource_type: self.resource_type.value().unwrap_or_default(),
            resource_id: self.resource_id.value().unwrap_or_default(),
            action: self.action.value().unwrap_or_default(),
            allow: self.allow.value().unwrap_or_default(),
            comment: self.comment.value().unwrap_or_default(),
        };

        api.send(AgentMessage::Request(ServerRequest::IAM(
            IAMRequest::PermissionStatementSave(statement),
        )));
    }

    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled {
        match response {
            ServerResponse::IAM(response) => match response {
                IAMResponse::PermissionStatement(statement) => {
                    if let Some(id) = &statement.id {
                        self.id.update(Some(*id));

                        self.service.update(statement.service);
                        self.resource_type.update(statement.resource_type);
                        self.resource_id.update(statement.resource_id);
                        self.action.update(statement.action);
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
        errors: Option<Rc<ErrorMap<Self::Fields>>>,
    ) -> Html {
        let is_new = edit_form.props.editing_id.is_new();
        let id = match edit_form.props.editing_id {
            EditingId::Id(_) => {
                html! {
                    <Field<PermissionStatementFields> field=PermissionStatementFields::Id errors=errors.clone()>
                        <Label text=PermissionStatementFields::Id.localized_name() />
                        <TextInput<PermissionStatementFields, i64> field=PermissionStatementFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
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
                        <TextInput<PermissionStatementFields, String> field=PermissionStatementFields::Service storage=self.service.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-service") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::ResourceType errors=errors.clone()>
                        <Label text=PermissionStatementFields::ResourceType.localized_name() />
                        <TextInput<PermissionStatementFields, String> field=PermissionStatementFields::ResourceType storage=self.resource_type.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-resource-type") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::ResourceId errors=errors.clone()>
                        <Label text=PermissionStatementFields::ResourceId.localized_name() />
                        <TextInput<PermissionStatementFields, i64> field=PermissionStatementFields::ResourceId storage=self.resource_id.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-resource-id") errors=errors.clone() />
                    </Field<PermissionStatementFields>>

                    <Field<PermissionStatementFields> field=PermissionStatementFields::Action errors=errors.clone()>
                        <Label text=PermissionStatementFields::Action.localized_name() />
                        <TextInput<PermissionStatementFields, String> field=PermissionStatementFields::Action storage=self.action.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("any-action") errors=errors.clone() />
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
                        <TextInput<PermissionStatementFields,String> field=PermissionStatementFields::Comment storage=self.comment.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder=localize!("permission-statement-comment-placeholder") errors=errors.clone() />
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
        info!("Self.resource_id: {:#?}", self.resource_id);
        ModelValidator::default()
            .with_field(
                PermissionStatementFields::ResourceId,
                self.resource_id.clone(),
            )
            .with_custom(
                PermissionStatementFields::ResourceId,
                self.resource_id
                    .is_absent()
                    .or(self.resource_type.is_present()),
                "permission-statements-error-resource-id-without-type",
            )
            .validate()
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

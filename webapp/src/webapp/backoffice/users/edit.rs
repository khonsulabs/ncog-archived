use crate::webapp::{
    api::ApiBridge,
    backoffice::{
        edit_form::{EditForm, Form, Handled, Message, Props},
        {entity_list::EntityList, users::fields::UserFields},
    },
    strings::LocalizableName,
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use shared::{
    iam::{users_read_claim, users_update_claim, IAMRequest, IAMResponse, RoleSummary},
    permissions::Claim,
    ServerRequest, ServerResponse,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct User {
    id: Rc<RefCell<String>>,
    screenname: Rc<RefCell<String>>,
    roles: Option<Vec<RoleSummary>>,
}

impl Form for User {
    type Fields = UserFields;

    fn title() -> &'static str {
        "edit-user"
    }

    fn load_request(&self, props: &Props) -> Option<ServerRequest> {
        props
            .editing_id
            .map(|account_id| shared::ServerRequest::IAM(IAMRequest::UsersGetProfile(account_id)))
    }

    fn save(&mut self, _props: &Props, _api: &mut ApiBridge) {
        todo!("need to be able to save users eventually")
    }

    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled {
        match response {
            ServerResponse::IAM(response) => match response {
                IAMResponse::UserProfile(profile) => {
                    if let Some(id) = &profile.id {
                        *self.id.borrow_mut() = id.to_string();
                        *self.screenname.borrow_mut() = profile.screenname.unwrap_or_default();
                        self.roles = Some(profile.roles);
                        Handled::ShouldRender(true)
                    } else {
                        Handled::ShouldRender(false)
                    }
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
        errors: Option<Rc<HashMap<Self::Fields, Vec<Rc<Html>>>>>,
    ) -> Html {
        html! {
            <div>
                <h2>{localize_html!("edit-user")}</h2>
                <form>
                    <flash::Flash message=edit_form.flash_message.clone() />
                    <Field<UserFields> field=UserFields::Id errors=errors.clone()>
                        <Label text=UserFields::Id.localized_name() />
                        <TextInput<UserFields> field=UserFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                    </Field<UserFields>>
                    <Field<UserFields> field=UserFields::Screenname errors=errors.clone()>
                        <Label text=UserFields::Screenname.localized_name() />
                        <TextInput<UserFields> field=UserFields::Screenname storage=self.screenname.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                    </Field<UserFields>>
                    <Button
                        label=localize!("save-user")
                        disabled=!can_save
                        css_class="is-primary"
                        action=edit_form.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                        processing=edit_form.is_saving
                    />
                </form>

                <h2>{UserFields::AssignedRoles.localized_name()}</h2>
                <EntityList<RoleSummary> entities=self.roles.clone() />
            </div>
        }
    }

    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>> {
        ModelValidator::new().validate()
    }

    fn read_claim(id: Option<i64>) -> Claim {
        users_read_claim(id)
    }
    fn update_claim(id: Option<i64>) -> Claim {
        users_update_claim(id)
    }
}

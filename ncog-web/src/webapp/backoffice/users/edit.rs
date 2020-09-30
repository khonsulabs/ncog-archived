use crate::webapp::{
    api::ApiBridge,
    backoffice::{
        edit_form::{EditForm, ErrorMap, Form, Handled, Message, Props},
        roles::summary_list,
        users::fields::UserFields,
    },
    strings::Namable,
    AppRoute, EditingId,
};
use khonsuweb::{flash, forms::prelude::*, validations::prelude::*};
use ncog_shared::{
    iam::{
        users_create_claim, users_read_claim, users_update_claim, IAMRequest, IAMResponse,
        RoleSummary,
    },
    permissions::Claim,
    NcogRequest, NcogResponse,
};
use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;

#[derive(Debug, Default)]
pub struct User {
    id: FormStorage<Option<i64>>,
    screenname: FormStorage<Option<String>>,
    roles: Option<Rc<RwLock<Vec<RoleSummary>>>>,
}

impl Form for User {
    type Message = ();
    type Fields = UserFields;

    fn title(is_new: bool) -> &'static str {
        if is_new {
            "add-user"
        } else {
            "edit-user"
        }
    }

    fn route_for(id: EditingId, _owning_id: Option<i64>) -> AppRoute {
        AppRoute::BackOfficeUserEdit(id)
    }

    fn load_request(&self, props: &Props) -> Option<NcogRequest> {
        props.editing_id.existing_id().map(|account_id| {
            ncog_shared::NcogRequest::IAM(IAMRequest::UsersGetProfile(account_id))
        })
    }

    fn save(&mut self, _props: &Props, _api: &mut ApiBridge) {
        todo!("need to be able to save users eventually")
    }

    fn handle_webserver_response(&mut self, response: NcogResponse) -> Handled {
        match response {
            NcogResponse::IAM(response) => match response {
                IAMResponse::UserProfile(profile) => {
                    if let Some(id) = &profile.id {
                        self.id.update(Some(*id));
                        self.screenname.update(profile.screenname);
                        self.roles = Some(Rc::new(RwLock::new(profile.roles)));
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
        errors: Option<Rc<ErrorMap<Self::Fields>>>,
    ) -> Html {
        html! {
            <div>
                <section class="section content">
                    <Title>{localize!("edit-user")}</Title>
                    <form>
                        <flash::Flash message=edit_form.flash_message.clone() />
                        <Field<UserFields> field=UserFields::Id errors=errors.clone()>
                            <Label text=UserFields::Id.localized_name() />
                            <TextInput<UserFields,i64> field=UserFields::Id storage=self.id.clone() readonly=true errors=errors.clone() />
                        </Field<UserFields>>
                        <Field<UserFields> field=UserFields::Screenname errors=errors.clone()>
                            <Label text=UserFields::Screenname.localized_name() />
                            <TextInput<UserFields,String> field=UserFields::Screenname storage=self.screenname.clone() readonly=readonly on_value_changed=edit_form.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                        </Field<UserFields>>
                        <Button
                            label=localize!("save-user")
                            disabled=!can_save
                            css_class="is-primary"
                            action=edit_form.link.callback(|e: web_sys::MouseEvent| {e.prevent_default(); Message::Save})
                            processing=edit_form.is_saving
                        />
                    </form>
                </section>

                <section class="Section content">
                    <Title size=3>{UserFields::AssignedRoles.localized_name()}</Title>

                    { summary_list::standard(self.roles.clone()) }

                </section>
            </div>
        }
    }

    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>> {
        ModelValidator::default().validate()
    }

    fn read_claim(id: Option<i64>) -> Claim {
        users_read_claim(id)
    }
    fn update_claim(id: Option<i64>) -> Claim {
        users_update_claim(id)
    }
    fn create_claim() -> Claim {
        users_create_claim()
    }
}

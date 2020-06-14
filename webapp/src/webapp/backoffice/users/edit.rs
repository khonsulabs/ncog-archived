use crate::{
    localize, localize_html, require_permission,
    webapp::{
        backoffice::users::UserFields, has_permission, strings::LocalizableName, LoggedInUser,
    },
};
use khonsuweb::{forms::prelude::*, validations::prelude::*};
use shared::permissions::Claim;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use yew::prelude::*;

pub struct EditUser {
    props: Props,
    user: User,
    link: ComponentLink<Self>,
}

#[derive(Debug)]
pub struct User {
    id: Rc<RefCell<String>>,
    screenname: Rc<RefCell<String>>,
}

pub enum Message {
    ValueChanged,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub editing_id: Option<i64>,
    pub set_title: Callback<String>,
}

impl Component for EditUser {
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let user = User {
            id: Rc::new(RefCell::new(String::new())),
            screenname: Rc::new(RefCell::new(String::new())),
        };
        Self { props, user, link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::ValueChanged => {
                info!("Current stored value: {}", self.user.screenname.borrow());
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        require_permission!(&self.props.user, read_claim(self.props.editing_id));

        let errors = self.validate().map(|errors| {
            errors.translate(|e| match e.error {
                ValidationError::NotPresent => {
                    localize_html!("form-field-required", "field" => e.primary_field().localized_name())
                }
            })
        });

        let readonly = !has_permission(&self.props.user, write_claim(self.props.editing_id));
        html! {
            <div>
                <h2>{localize_html!("edit-user")}</h2>
                <form>
                    <Field<UserFields> field=UserFields::Id errors=errors.clone()>
                        <Label text=UserFields::Id.localized_name() />
                        <TextInput<UserFields> field=UserFields::Id storage=self.user.id.clone() readonly=true errors=errors.clone() />
                    </Field<UserFields>>
                    <Field<UserFields> field=UserFields::Screenname errors=errors.clone()>
                        <Label text=UserFields::Screenname.localized_name() />
                        <TextInput<UserFields> field=UserFields::Screenname storage=self.user.screenname.clone() readonly=readonly on_value_changed=self.link.callback(|_| Message::ValueChanged) placeholder="Type your message here..." errors=errors.clone() />
                    </Field<UserFields>>
                //     <Button label="Send" disabled=disable_button css_class="is-success" action=self.link.callback(|e: ClickEvent| {e.prevent_default(); Message::SendMessage})/>
                </form>
            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        self.props.set_title.emit(localize!("edit-user"))
    }
}

pub fn read_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("users"), id, "read")
}

pub fn write_claim(id: Option<i64>) -> Claim {
    Claim::new("iam", Some("users"), id, "write")
}

impl EditUser {
    fn validate(&self) -> Option<Rc<ErrorSet<UserFields>>> {
        ModelValidator::new()
            .with_field(UserFields::Screenname, self.user.screenname.is_present())
            .validate()
    }
}

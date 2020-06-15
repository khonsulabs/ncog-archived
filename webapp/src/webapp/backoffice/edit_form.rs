use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    has_permission,
    strings::{LocalizableName, Namable},
    LoggedInUser,
};
use khonsuweb::{flash, validations::prelude::*};
use shared::{permissions::Claim, ServerRequest, ServerResponse};
use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};
use yew::prelude::*;

pub enum Handled {
    Saved(&'static str),
    ShouldRender(ShouldRender),
}
pub trait Form: Default {
    type Fields: Namable + Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static;

    fn title() -> &'static str;
    fn load_request(&self, props: &Props) -> Option<ServerRequest>;
    fn save(&mut self, props: &Props, api: &mut ApiBridge);
    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled;
    fn render(
        &self,
        edit_form: &EditForm<Self>,
        readonly: bool,
        can_save: bool,
        errors: Option<Rc<HashMap<Self::Fields, Vec<Rc<Html>>>>>,
    ) -> Html;
    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>>;
    fn read_claim(id: Option<i64>) -> Claim;
    fn update_claim(id: Option<i64>) -> Claim;
}

pub struct EditForm<T>
where
    T: Form + Default + 'static,
{
    api: ApiBridge,
    props: Props,
    form: T,
    pub link: ComponentLink<Self>,
    pub flash_message: Option<flash::Message>,
    pub is_saving: bool,
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

impl<T> Component for EditForm<T>
where
    T: Form + Default + 'static,
{
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let form = T::default();
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        Self {
            props,
            form,
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
                self.form.save(&self.props, &mut self.api);
                self.is_saving = true;
                true
            }
            Message::WsMessage(agent_response) => match agent_response {
                AgentResponse::Response(ws_response) => match ws_response.result {
                    ServerResponse::Error { message } => {
                        if let Some(message) = message {
                            self.flash_message = Some(flash::Message::new(
                                flash::Kind::Danger,
                                message,
                                Duration::from_secs(3),
                            ));
                        }
                        self.is_saving = false;
                        true
                    }
                    other => match self.form.handle_webserver_response(other) {
                        Handled::Saved(message) => self.saved(message),
                        Handled::ShouldRender(should_render) => should_render,
                    },
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
        require_permission!(&self.props.user, T::read_claim(self.props.editing_id));

        let errors = self.validate().map(|errors| {
            errors.translate(|e| match e.error {
                ValidationError::NotPresent => {
                    localize_html!("form-field-required", "field" => e.primary_field().localized_name())
                }
            })
        });

        let readonly = self.is_saving
            || !has_permission(&self.props.user, T::update_claim(self.props.editing_id));
        let can_save = !readonly && !errors.is_some();
        self.form.render(self, readonly, can_save, errors)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.initialize();
        }
        self.props.set_title.emit(localize!(T::title()))
    }
}

impl<T> EditForm<T>
where
    T: Form + Default + 'static,
{
    fn saved(&mut self, save_message: &'static str) -> ShouldRender {
        self.flash_message = Some(flash::Message::new(
            flash::Kind::Success,
            localize!(save_message),
            Duration::from_secs(5),
        ));
        self.is_saving = false;
        true
    }

    fn validate(&self) -> Option<Rc<ErrorSet<T::Fields>>> {
        self.form.validate()
    }

    fn initialize(&mut self) {
        match self.form.load_request(&self.props) {
            Some(request) => self.api.send(AgentMessage::Request(request)),
            None => {}
        }
    }
}

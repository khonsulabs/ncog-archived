use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    has_permission,
    strings::Namable,
    AppRoute, EditingId, LoggedInUser,
};
use khonsuweb::{flash, validations::prelude::*};
use shared::{permissions::Claim, ServerRequest, ServerResponse};
use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};
use yew::prelude::*;
use yew_router::{
    agent::{RouteAgentBridge, RouteRequest},
    route::Route,
};

pub enum Handled {
    Saved { label: &'static str, new_id: i64 },
    ShouldRender(ShouldRender),
}

pub type ErrorMap<K> = HashMap<K, Vec<Rc<Html>>>;

pub trait Form: Default {
    type Fields: Namable + Copy + std::hash::Hash + Eq + PartialEq + std::fmt::Debug + 'static;
    type Message: Clone + 'static;

    fn title(is_new: bool) -> &'static str;
    fn load_request(&self, props: &Props) -> Option<ServerRequest>;
    fn save(&mut self, props: &Props, api: &mut ApiBridge);
    fn handle_webserver_response(&mut self, response: ServerResponse) -> Handled;
    fn render(
        &self,
        edit_form: &EditForm<Self>,
        readonly: bool,
        can_save: bool,
        errors: Option<Rc<ErrorMap<Self::Fields>>>,
    ) -> Html;
    fn validate(&self) -> Option<Rc<ErrorSet<Self::Fields>>>;
    fn read_claim(id: Option<i64>) -> Claim;
    fn update_claim(id: Option<i64>) -> Claim;
    fn create_claim() -> Claim;
    fn route_for(id: EditingId, owning_id: Option<i64>) -> AppRoute;

    fn update(
        &mut self,
        _message: Self::Message,
        _props: &Props,
        _api: &mut ApiBridge,
    ) -> ShouldRender {
        false
    }
}

pub struct EditForm<T>
where
    T: Form + Default + 'static,
{
    api: ApiBridge,
    form: T,
    pub props: Props,
    pub link: ComponentLink<Self>,
    pub flash_message: Option<flash::Message>,
    pub is_saving: bool,
}
pub enum Message<T> {
    ValueChanged,
    Save,
    WsMessage(AgentResponse),
    FormMessage(T),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub user: Option<Arc<LoggedInUser>>,
    pub editing_id: EditingId,
    pub set_title: Callback<String>,
    #[prop_or_default]
    pub owning_id: Option<i64>,
}

impl<T> Component for EditForm<T>
where
    T: Form + Default + 'static,
{
    type Message = Message<T::Message>;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let form = T::default();
        let callback = link.callback(Message::WsMessage);
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
                        Handled::Saved { label, new_id } => self.saved(label, new_id),
                        Handled::ShouldRender(should_render) => should_render,
                    },
                },
                _ => false,
            },
            Message::FormMessage(form_message) => {
                self.form.update(form_message, &self.props, &mut self.api)
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        if self.props.editing_id.is_existing() {
            self.initialize()
        }
        true
    }

    fn view(&self) -> Html {
        if let Some(id) = self.props.editing_id.existing_id() {
            require_permission!(&self.props.user, T::read_claim(Some(id)));
        } else {
            require_permission!(&self.props.user, T::create_claim());
        }

        let errors = self.validate().map(|errors| {
            errors.translate(|e| match e.error {
                ValidationError::NotPresent => {
                    localize!("form-field-required", "field" => e.primary_field().localized_name())
                }
                ValidationError::InvalidValue => {
                    localize!("form-field-invalid-value", "field" => e.primary_field().localized_name())
                }
                ValidationError::Custom(message) => {
                    localize!(message)
                }
            })
        });

        let update_claim = match self.props.editing_id {
            EditingId::Id(id) => T::update_claim(Some(id)),
            EditingId::New => T::create_claim(),
        };
        let can_update = has_permission(&self.props.user, update_claim);
        let readonly = self.is_saving || !can_update;
        let can_save = !readonly && errors.is_none();
        self.form.render(self, readonly, can_save, errors)
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.initialize();
        }
        self.props
            .set_title
            .emit(localize!(T::title(self.props.editing_id.is_new())))
    }
}

impl<T> EditForm<T>
where
    T: Form + Default + 'static,
{
    fn saved(&mut self, save_message: &'static str, new_id: i64) -> ShouldRender {
        let new_id = EditingId::Id(new_id);
        let new_route = T::route_for(new_id, self.props.owning_id);
        let mut agent = RouteAgentBridge::<()>::new(Callback::noop());
        agent.send(RouteRequest::ReplaceRoute(Route::from(new_route)));
        self.flash_message = Some(flash::Message::new(
            flash::Kind::Success,
            localize!(save_message),
            Duration::from_secs(3),
        ));
        self.is_saving = false;
        true
    }

    fn validate(&self) -> Option<Rc<ErrorSet<T::Fields>>> {
        self.form.validate()
    }

    fn initialize(&mut self) {
        if let Some(request) = self.form.load_request(&self.props) {
            self.api.send(AgentMessage::Request(request))
        }
    }
}

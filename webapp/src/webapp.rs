#[macro_use]
mod strings;

use khonsuweb::static_page::StaticPage;
use loggedin::LoggedIn;
use login::Login;
use shared::{
    permissions::{Claim, PermissionSet},
    ServerResponse, UserProfile,
};
use std::sync::Arc;
use strings::localize;
use yew::prelude::*;
use yew_router::{agent::RouteRequest, prelude::*};

#[macro_export]
macro_rules! require_permission {
    ($user:expr, $claim:expr) => {{
        if !crate::webapp::has_permission($user, $claim) {
            return crate::webapp::invalid_permissions();
        }
    }};
}

mod api;
mod backoffice;
mod loggedin;
mod login;
use api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge};

pub struct App {
    link: ComponentLink<Self>,
    show_nav: Option<bool>,
    api: ApiBridge,
    _route_agent: RouteAgentBridge,
    connected: Option<bool>,
    user: Option<Arc<LoggedInUser>>,
    current_route: String,
}

#[derive(PartialEq, Debug)]
pub struct LoggedInUser {
    pub profile: UserProfile,
    pub permissions: PermissionSet,
}

#[derive(Debug)]
pub enum Message {
    SetTitle(String),
    ToggleNavgar,
    WsMessage(AgentResponse),
    RouteMessage(Route),
    LogOut,
}
#[derive(Switch, Clone, Debug)]
pub enum AppRoute {
    #[to = "/_dev/styles"]
    StylesTest,
    #[to = "/login!"]
    LogIn,
    #[to = "/auth/callback/{service}"]
    LoggedIn(String),
    #[to = "/backoffice/users/{id}"]
    BackOfficeUserEdit(i64),
    #[to = "/backoffice/users!"]
    BackOfficeUsersList,
    #[to = "/backoffice/roles/{id}"]
    BackOfficeRoleEdit(i64),
    #[to = "/backoffice/roles!"]
    BackOfficeRolesList,
    #[to = "/backoffice/permissions/{id}"]
    BackOfficeRolePermissionStatementEdit(i64),
    #[to = "/backoffice!"]
    BackOfficeDashboard,
    #[to = "/!"]
    Index,
    #[to = "/"]
    NotFound,
}
impl AppRoute {
    pub fn render(&self, set_title: Callback<String>, user: Option<Arc<LoggedInUser>>) -> Html {
        match self {
            AppRoute::Index => {
                html! {<StaticPage title="Welcome" content=localize("home-page") set_title=set_title.clone() />}
            }
            AppRoute::NotFound => {
                html! {<StaticPage title="Not Found" content=localize("not-found") set_title=set_title.clone() />}
            }
            AppRoute::StylesTest => style_test(),
            AppRoute::LogIn => html! {<Login />},
            AppRoute::LoggedIn(service) => html! {<LoggedIn service=service.clone() />},
            AppRoute::BackOfficeDashboard => {
                html! { <backoffice::Dashboard set_title=set_title.clone() user=user.clone() />}
            }
            AppRoute::BackOfficeUsersList => {
                html! { <backoffice::users::list::UsersList set_title=set_title.clone() user=user.clone() />}
            }
            AppRoute::BackOfficeUserEdit(id) => {
                html! { <backoffice::edit_form::EditForm<backoffice::users::edit::User> set_title=set_title.clone() user=user.clone() editing_id=Some(*id) /> }
            }
            AppRoute::BackOfficeRolesList => {
                html! { <backoffice::roles::list::RolesList set_title=set_title.clone() user=user.clone() />}
            }
            AppRoute::BackOfficeRoleEdit(id) => {
                html! { <backoffice::edit_form::EditForm<backoffice::roles::edit::Role> set_title=set_title.clone() user=user.clone() editing_id=Some(*id) /> }
            }
            AppRoute::BackOfficeRolePermissionStatementEdit(_id) => {
                todo!("need to add role permission statement view");
                html! { <p>{"todo"}</p> }
            }
        }
    }
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        let mut route_agent =
            RouteAgentBridge::new(link.callback(|message| Message::RouteMessage(message)));
        route_agent.send(RouteRequest::GetCurrentRoute);
        App {
            link,
            show_nav: None,
            api,
            user: None,
            connected: None,
            _route_agent: route_agent,
            current_route: "/".to_owned(),
        }
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::SetTitle(title) => {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        document.set_title(&format!("{} - ncog.link", title));
                    }
                }
                false
            }
            Message::ToggleNavgar => {
                self.show_nav = Some(!self.show_nav.unwrap_or(false));
                true
            }
            Message::WsMessage(message) => match message {
                AgentResponse::Disconnected => {
                    self.user = None;
                    self.connected = Some(false);
                    true
                }
                AgentResponse::Connected => {
                    self.user = None;
                    self.connected = Some(true);
                    true
                }
                AgentResponse::Response(response) => match response.result {
                    ServerResponse::Authenticated {
                        profile,
                        permissions,
                    } => {
                        self.user = Some(Arc::new(LoggedInUser {
                            profile,
                            permissions,
                        }));
                        true
                    }
                    _ => false,
                },
                _ => false,
            },
            Message::RouteMessage(route) => {
                self.current_route = route.route;
                info!("New route: {}", self.current_route);
                true
            }
            Message::LogOut => {
                self.api.send(AgentMessage::LogOut);
                false
            }
        }
    }

    fn view(&self) -> Html {
        let set_title = self.link.callback(Message::SetTitle);
        let user = self.user.clone();
        html! {
            <div>
                { self.nav_bar() }

                <section class="section content">
                    <div class="columns is-centered">
                        <div class="column is-half">
                            <p class="notification is-danger is-light">
                                { localize("early-warning") }
                            </p>
                        </div>
                    </div>
                    <Router<AppRoute>
                        render = Router::render(move |switch: AppRoute| {
                            switch.render(set_title.clone(), user.clone())
                        })
                    />
                </section>

                { self.footer() }
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.api.send(AgentMessage::RegisterBroadcastHandler);
            self.api.send(AgentMessage::Initialize);
        }
    }

    fn destroy(&mut self) {
        self.api.send(AgentMessage::UnregisterBroadcastHandler);
    }
}

impl App {
    fn navbar_class(&self) -> &'static str {
        match self.show_nav {
            Some(state) => {
                if state {
                    return "navbar-menu is-active";
                }
            }
            None => {}
        }
        "navbar-menu"
    }

    fn navbar_class_for(&self, default_class: &'static str, path: &'static str) -> String {
        if self.current_route.starts_with(path) {
            // Special case "/" -- make it be an exact match, not a starts_with
            if path.len() > 1 || self.current_route.len() == 1 {
                return format!("{} is-active", default_class);
            }
        }

        default_class.to_owned()
    }

    fn nav_bar(&self) -> Html {
        let toggle_navbar = self.link.callback(|_| Message::ToggleNavgar);
        let backoffice_links = if has_permission(&self.user, backoffice::read_claim()) {
            html! {
                <div class="navbar-item has-dropdown is-hoverable">
                    <RouterAnchor<AppRoute> route=AppRoute::BackOfficeDashboard classes=self.navbar_class_for("navbar-link", "/backoffice") >{ localize("backoffice") }</RouterAnchor<AppRoute>>
                    <div class="navbar-dropdown is-boxed">
                        <RouterAnchor<AppRoute> route=AppRoute::BackOfficeUsersList classes=self.navbar_class_for("navbar-item", "/backoffice/users") >{ localize("users") }</RouterAnchor<AppRoute>>
                        <RouterAnchor<AppRoute> route=AppRoute::BackOfficeRolesList classes=self.navbar_class_for("navbar-item", "/backoffice/roles") >{ localize("roles") } </RouterAnchor<AppRoute>>
                    </div>
                </div>
            }
        } else {
            html! {}
        };
        // let player_name = match &self.current_avatar {
        //     Some(avatar) => {
        //         html! {<RouterAnchor<AppRoute> route=AppRoute::AvatarProfile(avatar.name.clone()) classes="navbar-item" >{ &avatar.name } </RouterAnchor<AppRoute>>}
        //     }
        //     None => html! {},
        // };
        html! {
            <nav class="navbar is-dark" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                <RouterAnchor<AppRoute> route=AppRoute::Index classes="navbar-item" ><h1 class="is-size-3 has-text-primary">{ "ncog.link" }</h1></RouterAnchor<AppRoute>>
                <a role="button" class="navbar-burger burger" aria-label="menu" aria-expanded="false" data-target="navbarMenu" onclick=toggle_navbar.clone()>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
                </div>
                <div id="navbarMenu" class=self.navbar_class()>
                    <div class="navbar-start">
                        <RouterAnchor<AppRoute> route=AppRoute::Index classes=self.navbar_class_for("navbar-item", "/") >{ localize("home") } </RouterAnchor<AppRoute>>
                        { backoffice_links }
                    </div>
                    <div class="navbar-end">
                        { self.login_button() }
                        { self.connection_indicator() }
                    </div>
                </div>
            </nav>
        }
    }

    fn footer(&self) -> Html {
        html! {
            <footer class="footer">
                <div class="content has-text-centered">
                    { localize("footer") }
                </div>
            </footer>
        }
    }

    fn login_button(&self) -> Html {
        if let Some(user) = &self.user {
            html! {
                <div class="navbar-item">
                    { user.profile.screenname.clone().unwrap_or_default() }
                    <button class="button" onclick=self.link.callback(|_| Message::LogOut)>
                        <strong>{ localize("log-out") }</strong>
                    </button>
                </div>
            }
        } else {
            html! {
                <div class="navbar-item">
                    <RouterButton<AppRoute> route=AppRoute::LogIn classes="button is-primary" >
                        <strong>{ localize("log-in") }</strong>
                    </RouterButton<AppRoute>>
                </div>
            }
        }
    }

    fn connection_indicator(&self) -> Html {
        if let Some(connected) = &self.connected {
            if *connected {
                html! {
                    <div class="navbar-item">
                        <span class="has-text-success"><i class="fas fa-circle" alt="Connected"></i></span>
                    </div>
                }
            } else {
                html! {
                    <div class="navbar-item">
                        <span class="has-text-danger"><i class="fas fa-times-circle" alt="Not able to connect"></i></span>
                    </div>
                }
            }
        } else {
            html! {
                <div class="navbar-item">
                    <span class="has-text-info"><i class="fas fa-circle" alt="Connecting..."></i></span>
                </div>
            }
        }
    }
}

fn style_test() -> Html {
    html! {
        <div>
            <h1>{"Heading 1"}<a href="#">{"Heading Link"}</a></h1>
            <h2>{"Heading 2"}<a href="#">{"Heading Link"}</a></h2>
            <h3>{"Heading 3"}<a href="#">{"Heading Link"}</a></h3>
            <h4>{"Heading 4"}<a href="#">{"Heading Link"}</a></h4>
            <h5>{"Heading 5"}<a href="#">{"Heading Link"}</a></h5>
            <h6>{"Heading 6"}<a href="#">{"Heading Link"}</a></h6>
            <h1>{"Buttons - Normal"}</h1>
            <a href="#" class="button" >{"Plain"}</a>
            <a href="#" class="button is-text" >{"Text"}</a>
            <a href="#" class="button is-primary" >{"Primary"}</a>
            <a href="#" class="button is-link" >{"Link"}</a>
            <a href="#" class="button is-info" >{"Info"}</a>
            <a href="#" class="button is-success" >{"Success"}</a>
            <a href="#" class="button is-warning" >{"Warning"}</a>
            <a href="#" class="button is-danger" >{"Danger"}</a>
            <h1>{"Buttons - Light"}</h1>
            <a href="#" class="button  is-light" >{"Plain"}</a>
            <a href="#" class="button is-text is-light" >{"Text"}</a>
            <a href="#" class="button is-primary is-light" >{"Primary"}</a>
            <a href="#" class="button is-link is-light" >{"Link"}</a>
            <a href="#" class="button is-info is-light" >{"Info"}</a>
            <a href="#" class="button is-success is-light" >{"Success"}</a>
            <a href="#" class="button is-warning is-light" >{"Warning"}</a>
            <a href="#" class="button is-danger is-light" >{"Danger"}</a>
            <h1>{"Buttons - Inverted"}</h1>
            <a href="#" class="button is-inverted" >{"Plain"}</a>
            <a href="#" class="button is-text is-inverted" >{"Text"}</a>
            <a href="#" class="button is-primary is-inverted" >{"Primary"}</a>
            <a href="#" class="button is-link is-inverted" >{"Link"}</a>
            <a href="#" class="button is-info is-inverted" >{"Info"}</a>
            <a href="#" class="button is-success is-inverted" >{"Success"}</a>
            <a href="#" class="button is-warning is-inverted" >{"Warning"}</a>
            <a href="#" class="button is-danger is-inverted" >{"Danger"}</a>
        </div>
    }
}

fn has_permission(user: &Option<Arc<LoggedInUser>>, claim: Claim) -> bool {
    if let Some(user) = user {
        user.permissions.allowed(&claim)
    } else {
        false
    }
}

fn invalid_permissions() -> Html {
    html! {
        <div class="notification is-danger">
            {localize("no-permission")}
        </div>
    }
}

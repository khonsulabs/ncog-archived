use crate::strings::prelude::*;
use khonsuweb::static_page::StaticPage;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::{
    agent::{RouteAgentDispatcher, RouteRequest},
    prelude::*,
};
pub struct App {
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher,
    show_nav: Option<bool>,
}

#[derive(Debug)]
pub enum Message {
    SetTitle(String),
    ToggleNavgar,
}
#[derive(Switch, Clone, Debug, Serialize, Deserialize)]
pub enum AppRoute {
    #[to = "/"]
    Index,
}
impl AppRoute {
    pub fn render(&self, set_title: Callback<String>) -> Html {
        match self {
            AppRoute::Index => {
                html! {<StaticPage title="Welcome" content=localize("markdown/index.md") set_title=set_title.clone() />}
            }
        }
    }
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::default();
        App {
            link,
            router,
            show_nav: None,
        }
    }

    fn change(&mut self, _: Self::Properties) -> bool {
        false
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let set_title = self.link.callback(Message::SetTitle);
        html! {
            <div>
                { self.nav_bar() }

                <section class="section content">
                    <div class="columns is-centered">
                        <div class="column is-half">
                            <p class="notification is-danger">
                                { "ncog.link is extremely early in development. Nothing is set in stone. To join in on the discussion, head over to "}
                                <a href="https://community.khonsulabs.com/">{"the forums"}
                                </a>{"."}
                            </p>
                        </div>
                    </div>
                    <Router<AppRoute>
                        render = Router::render(move |switch: AppRoute| {
                            switch.render(set_title.clone())
                        })
                    />
                </section>

                { self.footer() }
            </div>
        }
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

    fn nav_bar(&self) -> Html {
        let toggle_navbar = self.link.callback(|_| Message::ToggleNavgar);
        // let player_name = match &self.current_avatar {
        //     Some(avatar) => {
        //         html! {<RouterAnchor<AppRoute> route=AppRoute::AvatarProfile(avatar.name.clone()) classes="navbar-item" >{ &avatar.name } </RouterAnchor<AppRoute>>}
        //     }
        //     None => html! {},
        // };
        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                <a class="navbar-item" href="/">
                    <h1 class="is-size-3 has-text-primary">{ "ncog.link" }</h1>
                </a>
                <a role="button" class="navbar-burger burger" aria-label="menu" aria-expanded="false" data-target="navbarMenu" onclick=toggle_navbar.clone()>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
                </div>
                <div id="navbarMenu" class=self.navbar_class()>
                    <div class="navbar-start">
                        <RouterAnchor<AppRoute> route=AppRoute::Index classes="navbar-item" >{ "Home" } </RouterAnchor<AppRoute>>
                    </div>
                    // <div class="navbar-end">
                    //     { player_name }
                    //     { self.login_button() }
                    // </div>
                </div>
            </nav>
        }
    }

    fn footer(&self) -> Html {
        html! {
            <footer class="footer">
                <div class="content has-text-centered">
                    { localize("markdown/footer.md") }
                </div>
            </footer>
        }
    }
}

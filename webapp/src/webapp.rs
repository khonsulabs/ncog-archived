use crate::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    strings::prelude::*,
};
use khonsuweb::static_page::StaticPage;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;
pub struct App {
    link: ComponentLink<Self>,
    show_nav: Option<bool>,
    api: ApiBridge,
}

#[derive(Debug)]
pub enum Message {
    SetTitle(String),
    ToggleNavgar,
    WsMessage(AgentResponse),
}
#[derive(Switch, Clone, Debug, Serialize, Deserialize)]
pub enum AppRoute {
    #[to = "/_dev/styles"]
    StylesTest,
    #[to = "/"]
    Index,
}
impl AppRoute {
    pub fn render(&self, set_title: Callback<String>) -> Html {
        match self {
            AppRoute::Index => {
                html! {<StaticPage title="Welcome" content=localize("markdown/index.md") set_title=set_title.clone() />}
            }
            AppRoute::StylesTest => style_test(),
        }
    }
}

impl Component for App {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        App {
            link,
            show_nav: None,
            api,
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
                        document.set_title(&title);
                    }
                }
                false
            }
            Message::ToggleNavgar => {
                self.show_nav = Some(!self.show_nav.unwrap_or(false));
                true
            }
            Message::WsMessage(message) => {
                web_sys::console::info_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "Received response: {:?}",
                    message
                )));
                false
            }
        }
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

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.api.send(AgentMessage::RegisterBroadcastHandler);
            self.api.send(AgentMessage::Initialize);
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

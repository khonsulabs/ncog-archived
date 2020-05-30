use crate::api::OAuthProvider;
use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
}
pub enum Message {
    LogInWith(OAuthProvider),
}

impl Component for Login {
    type Message = Message;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogInWith(provider) => todo!(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html!(
            <div class="login columns is-centered">
                <div class="column is-half">
                    <h1>{"Sign up/ Log in"}</h1>
                    <p>{"ncog.link leverages the security and trust of major providers for logging in. Instead of needing to remember another password, simply use one of the providers below to log in or create an account."}</p>
                    <button class="button is-primary itchio-button" onclick=self.link.callback(|_| Message::LogInWith(OAuthProvider::ItchIO))>
                        {"Log in with"} <img alt="itch.io" src="/providers/itchio.svg" />
                    </button>
                </div>
            </div>
        )
    }
}

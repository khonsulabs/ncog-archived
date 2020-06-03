use crate::api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge};
use shared::{OAuthProvider, ServerRequest};
use yew::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    api: ApiBridge,
    current_storage_status: bool,
}
pub enum Message {
    LogInWith(OAuthProvider),
    ApiMessage(AgentResponse),
    ToggleStatus,
}

impl Component for Login {
    type Message = Message;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut api = ApiAgent::bridge(link.callback(|msg| Message::ApiMessage(msg)));
        api.send(AgentMessage::QueryStorageStatus);
        Self {
            link,
            api,
            current_storage_status: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogInWith(provider) => {
                self.api
                    .send(AgentMessage::Request(ServerRequest::AuthenticationUrl(
                        provider,
                    )));
                false
            }
            Message::ApiMessage(msg) => match msg {
                AgentResponse::StorageStatus(status) => {
                    self.current_storage_status = status;
                    true
                }
                _ => false,
            },
            Message::ToggleStatus => {
                self.api.send(if self.current_storage_status {
                    AgentMessage::DisableStorage
                } else {
                    AgentMessage::EnableStorage
                });
                false
            }
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
                    <div class="notification is-info has-text-left">
                        <label class="checkbox">
                            {"Signing in requires that ncog.link utilize Local Storage which is a technology similar to Cookies. We do not currently have a privacy policy as this product is so early in development."}
                            <br />
                            <br />
                            <input type="checkbox" checked=self.current_storage_status onclick=self.link.callback(|_| Message::ToggleStatus) />
                            {" By checking this checkbox, you permit ncog.link to store information in your browser that will be used to keep you logged in. You acknowledge that this software is under development and any and all data stored by ncog.link may be erased at any time. Khonsu Labs offers no warranties or guarantees on the functionality of this service at this time. Full international privacy law complaince will be part of ncog.link as development proceeds, but at this point in time it is too early to engage legal council over decisions that have yet to be made."}
                        </label>
                    </div>
                    <button class="button is-primary itchio-button" disabled=!self.current_storage_status onclick=self.link.callback(|_| Message::LogInWith(OAuthProvider::ItchIO))>
                        {"Log in with"} <img alt="itch.io" src="/providers/itchio.svg" />
                    </button>
                </div>
            </div>
        )
    }
}

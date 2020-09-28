use crate::webapp::{
    api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge},
    strings::{localize, localize_raw},
};
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
        let mut api = ApiAgent::bridge(link.callback(Message::ApiMessage));
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
                    <h1>{localize("log-in")}</h1>
                    <p>{localize("login-intro")}</p>
                    <div class="notification is-info has-text-left">
                        <label class="checkbox">
                            {localize("storage-agreement")}
                            <br />
                            <br />
                            <input type="checkbox" checked=self.current_storage_status onclick=self.link.callback(|_| Message::ToggleStatus) />
                            {localize_raw("i-agree")}
                        </label>
                    </div>
                    <button class="button twitch-button" disabled=!self.current_storage_status onclick=self.link.callback(|_| Message::LogInWith(OAuthProvider::Twitch))>
                        {localize("log-in-with-twitch")}
                    </button>
                </div>
            </div>
        )
    }
}

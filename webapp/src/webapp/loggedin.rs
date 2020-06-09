use super::api::{AgentMessage, AgentResponse, ApiAgent, ApiBridge};
use shared::ServerRequest;
use yew::prelude::*;

pub struct LoggedIn {
    api: ApiBridge,
    props: Props,
}
#[derive(Debug)]
pub enum Message {
    WsMessage(AgentResponse),
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub service: String,
}

impl Component for LoggedIn {
    type Message = Message;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|message| Message::WsMessage(message));
        let api = ApiAgent::bridge(callback);
        Self { api, props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::WsMessage(response) => match response {
                AgentResponse::Connected => {
                    info!("LoggedIn::Connected");
                    let window = web_sys::window().expect("Need a window");
                    if self.props.service == "itchio" {
                        let hash = window.location().hash().expect(
                            "itchio always passes its parameters after #, so this should always be present",
                        );
                        let params = web_sys::UrlSearchParams::new_with_str(&hash[1..hash.len()])
                            .expect("Couldn't parse parameters from itchio");
                        let access_token = params
                            .get("access_token")
                            .expect("access_token was not present");
                        let state = params.get("state").expect("state was not present");

                        self.api
                            .send(AgentMessage::Request(ServerRequest::ReceiveItchIOAuth {
                                access_token,
                                state,
                            }));
                    } else {
                        error!("Invalid service on OAuth callback: {}", self.props.service);
                    }
                }
                _ => {}
            },
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.api.send(AgentMessage::RegisterBroadcastHandler);
        }
    }

    fn view(&self) -> Html {
        html!(
            <div class="columns is-centered">
                <div class="column is-half">
                    <h1>{"Signing in..."}</h1>
                    <progress class="progress is-primary" max="100"/>
                </div>
            </div>
        )
    }

    fn destroy(&mut self) {
        self.api.send(AgentMessage::UnregisterBroadcastHandler);
    }
}

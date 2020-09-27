use crate::wasm_utc_now;
use chrono::{DateTime, Utc};
use std::time::Duration;
use yew::prelude::*;
use yew::services::timeout::{TimeoutService, TimeoutTask};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Kind {
    Primary,
    Link,
    Info,
    Success,
    Warning,
    Danger,
}

impl Kind {
    pub fn css_class(&self) -> &'static str {
        match self {
            Kind::Primary => "is-primary",
            Kind::Link => "is-link",
            Kind::Info => "is-info",
            Kind::Success => "is-success",
            Kind::Warning => "is-warning",
            Kind::Danger => "is-danger",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub created_at: DateTime<Utc>,
    pub kind: Kind,
    pub content: String,
    pub duration: Duration,
}

impl Message {
    pub fn new<S: Into<String>, D: Into<Duration>>(kind: Kind, content: S, duration: D) -> Self {
        Message {
            kind,
            created_at: wasm_utc_now(),
            content: content.into(),
            duration: duration.into(),
        }
    }
}

#[derive(Debug)]
pub struct Flash {
    link: ComponentLink<Self>,
    props: Props,
    timeout: TimeoutService,
    hide_task: Option<TimeoutTask>,
}

#[derive(Debug, Clone, Properties)]
pub struct Props {
    #[prop_or_default]
    pub message: Option<Message>,
}

pub enum ComponentMessage {
    Hide,
}

impl Component for Flash {
    type Message = ComponentMessage;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            timeout: TimeoutService::default(),
            hide_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ComponentMessage::Hide => true,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.message != self.props.message {
            if let Some(new_message) = &props.message {
                self.hide_task = Some(TimeoutService::spawn(
                    new_message.duration,
                    self.link.callback(|_| ComponentMessage::Hide),
                ));
            }
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        if let Some(message) = &self.props.message {
            let should_show = wasm_utc_now()
                < message
                    .created_at
                    .checked_add_signed(
                        chrono::Duration::from_std(message.duration).expect("Invalid duration"),
                    )
                    .expect("Unexpected date math error");
            if should_show {
                return html! {
                    <div class=format!("notification {}", message.kind.css_class())>
                        { &message.content }
                    </div>
                };
            }
        }

        Html::default()
    }
}

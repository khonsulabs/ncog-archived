use basws_client::prelude::*;
use ncog_shared::{ncog_protocol_version, AuthenticatedUser, NcogRequest, NcogResponse};

pub type NcogClient<T> = Client<Ncog<T>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("protocol error")]
    Protocol(#[from] basws_client::Error),
    #[error("browser error")]
    WebBrowser(#[from] std::io::Error),
    #[error("error, message: {0:?}")]
    Generic(Option<String>),
}

#[async_trait]
pub trait NcogClientLogic: Send + Sync + Sized + 'static {
    async fn handle_error(&self, error: Error, client: NcogClient<Self>) -> anyhow::Result<()>;
    async fn stored_installation_config(&self) -> Option<InstallationConfig>;
    async fn store_installation_config(&self, config: InstallationConfig) -> anyhow::Result<()>;
    async fn handle_response(
        &self,
        response: NcogResponse,
        original_request_id: Option<u64>,
        client: NcogClient<Self>,
    ) -> anyhow::Result<()>;

    fn connect_to_local_websocket(&self) -> bool {
        cfg!(debug_assertions)
    }

    async fn state_changed(
        &self,
        client: NcogClient<Self>,
        auth_state: AuthState,
    ) -> anyhow::Result<()>;

    async fn authenticated(&self, _client: NcogClient<Self>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn disconnected(&self, _client: NcogClient<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct Ncog<T> {
    pub logic: T,
    auth_state: Handle<AuthState>,
}

#[async_trait]
impl<T> ClientLogic for Ncog<T>
where
    T: NcogClientLogic,
{
    type Request = NcogRequest;
    type Response = NcogResponse;

    fn server_url(&self) -> Url {
        if self.logic.connect_to_local_websocket() {
            Url::parse("ws://localhost:7878/v1/ws").unwrap()
        } else {
            Url::parse("wss://api.ncog.id/v1/ws").unwrap()
        }
    }

    fn protocol_version(&self) -> Version {
        ncog_protocol_version()
    }

    async fn state_changed(&self, state: &LoginState, client: Client<Self>) -> anyhow::Result<()> {
        match state {
            LoginState::Connected { .. } => self.set_auth_state(AuthState::Connected, client).await,
            LoginState::Disconnected => self.set_auth_state(AuthState::LoggedOut, client).await,
            LoginState::Handshaking { .. } => Ok(()),
            LoginState::Error { message } => {
                self.set_auth_state(
                    AuthState::Error {
                        message: message.clone(),
                    },
                    client,
                )
                .await
            }
        }
    }

    async fn stored_installation_config(&self) -> Option<InstallationConfig> {
        self.logic.stored_installation_config().await
    }

    async fn store_installation_config(&self, config: InstallationConfig) -> anyhow::Result<()> {
        self.logic.store_installation_config(config).await
    }

    async fn response_received(
        &self,
        response: Self::Response,
        original_request_id: Option<u64>,
        client: Client<Self>,
    ) -> anyhow::Result<()> {
        match response {
            NcogResponse::Error { message } => {
                self.logic
                    .handle_error(Error::Generic(message), client)
                    .await
            }
            NcogResponse::Authenticated(user) => {
                self.set_auth_state(AuthState::Authenticated(user), client)
                    .await
            }
            NcogResponse::AuthenticateAtUrl { url } => {
                if let Err(err) = webbrowser::open(&url) {
                    self.logic.handle_error(Error::from(err), client).await
                } else {
                    Ok(())
                }
            }
            unhandled => {
                self.logic
                    .handle_response(unhandled, original_request_id, client)
                    .await
            }
        }
    }

    async fn handle_error(
        &self,
        error: basws_client::Error,
        client: Client<Self>,
    ) -> anyhow::Result<()> {
        self.logic
            .handle_error(Error::Protocol(error), client)
            .await
    }
}

impl<T> Ncog<T>
where
    T: NcogClientLogic,
{
    pub fn new(logic: T) -> Self {
        Self {
            logic,
            auth_state: Handle::new(AuthState::LoggedOut),
        }
    }

    async fn set_auth_state(
        &self,
        new_state: AuthState,
        client: NcogClient<T>,
    ) -> anyhow::Result<()> {
        {
            let mut auth_state = self.auth_state.write().await;
            *auth_state = new_state.clone();
        }
        self.logic.state_changed(client, new_state).await
    }

    pub async fn auth_state(&self) -> AuthState {
        let auth_state = self.auth_state.read().await;
        auth_state.clone()
    }
}

#[derive(Clone, Debug)]
pub enum AuthState {
    LoggedOut,
    Connected,
    Authenticated(AuthenticatedUser),
    Error { message: Option<String> },
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState::LoggedOut
    }
}

impl AuthState {
    pub fn is_connected(&self) -> bool {
        matches!(self, AuthState::Connected) || matches!(self, AuthState::Authenticated(_))
    }
}

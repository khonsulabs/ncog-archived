use super::{database, twitch};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use ncog_migrations::pg;
use ncog_shared::{
    jsonwebtoken::{self, EncodingKey},
    jwk::JwtKey,
    ncog_protocol_version_requirements,
    permissions::{Claim, PermissionSet},
    IdentityVerificationClaims, NcogRequest, NcogResponse, OAuthProvider, UserProfile,
};
use uuid::Uuid;
mod iam;
use basws_server::prelude::*;

#[async_trait]
pub trait ConnectedAccountHandle {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error>;
}

fn permission_denied(claim: &Claim) -> Result<(), anyhow::Error> {
    anyhow::bail!("permission denied for accessing {:?}", claim)
}

#[async_trait]
impl ConnectedAccountHandle for ConnectedClient<NcogServer> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        if let Some(account) = self.account().await {
            return account.permission_allowed(claim).await;
        }

        permission_denied(claim)
    }
}

#[async_trait]
impl ConnectedAccountHandle for Handle<ConnectedAccount> {
    async fn permission_allowed(&self, claim: &Claim) -> Result<(), anyhow::Error> {
        let account = self.read().await;
        if account.permissions.allowed(&claim) {
            Ok(())
        } else {
            permission_denied(claim)
        }
    }
}

#[derive(Debug)]
pub struct ConnectedAccount {
    pub profile: UserProfile,
    pub permissions: PermissionSet,
}

impl ConnectedAccount {
    pub async fn lookup(installation_id: Uuid) -> anyhow::Result<Self> {
        let profile = database::get_profile_by_installation_id(&pg(), installation_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no profile found"))?;
        let permissions = database::load_permissions_for(&pg(), profile.id).await?;
        Ok(Self {
            profile,
            permissions,
        })
    }
}

impl Identifiable for ConnectedAccount {
    type Id = i64;
    fn id(&self) -> Self::Id {
        self.profile.id
    }
}
pub struct NcogServer;

pub fn initialize() -> Server<NcogServer> {
    Server::new(NcogServer)
}

#[async_trait]
impl ServerLogic for NcogServer {
    type Request = NcogRequest;
    type Response = NcogResponse;
    type Client = ();
    type Account = ConnectedAccount;
    type AccountId = i64;

    async fn handle_request(
        &self,
        client: &ConnectedClient<Self>,
        request: Self::Request,
        _server: &Server<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        match request {
            NcogRequest::AuthenticationUrl(provider) => match provider {
                OAuthProvider::Twitch => {
                    if let Some(installation) = client.installation().await {
                        Ok(RequestHandling::Respond(NcogResponse::AuthenticateAtUrl {
                            url: twitch::authorization_url(installation.id),
                        }))
                    } else {
                        anyhow::bail!("Requested authentication URL without being connected")
                    }
                }
            },
            NcogRequest::IAM(iam_request) => iam::handle_request(client, iam_request).await,
            NcogRequest::ListPublicJwtKeys => {
                Ok(RequestHandling::Respond(NcogResponse::JwtPublicKeys(vec![
                                JwtKey {
                                    key_id: "sig-1602646606".to_string(),
                                    key_type: "RSA".to_string(),
                                    public_use: "sig".to_string(),
                                    rsa_e: "AQAB".to_string(),
                                    rsa_n: "uJx3D72SGsVZWny06jHms1o0TE8NMn2xDuBODnFzZbTxp0M_KXfPRZbW0fTVm3NRmaZ9c1MgQI0TyZ-zNw832dkNuHUaIXzRFWD2OuuRWmpHwfjMN1o6RZ5phfzJcbd1Qf0AmZLsAAIWSlPvWrow2GOIizqLcVokHx0dWkeMDQrWnLIQmcYKG5P18HpJPg_51X7A301ejavo91CZfWmKdHPN8If6ZB1koMJ8CAqPbfAtkapmrHZ6AA7m2fKNsVukKHzeqlYpEFcVEjWFIjZvtl8zz6y2ZXpsW7whu7MP816Jn0pOmaPmll41U7jbx703x6hiobozeK_QNaipCquFhQ".to_string(),
                                    algorithm: "RS256".to_string(),
                                }])))
            }
            NcogRequest::RequestIdentityVerificationToken { nonce, audience } => {
                if let Some(account) = client.account().await {
                    let account = account.read().await;
                    let encoding_key = EncodingKey::from_rsa_pem(std::env!("JWK_RSA_PRIVATE_KEY_PEM").as_bytes())?;
                    let issuance_time = Utc::now();
                    let expiration_time = issuance_time.checked_add_signed(Duration::minutes(5)).unwrap();
                    let issuance_time = issuance_time.timestamp() as u64;
                    let expiration_time = expiration_time.timestamp() as u64;
                    let subject = account.profile.id.to_string();
                    let claims = IdentityVerificationClaims {
                        issuer: "https://ncog.id/v1/ws".to_string(),
                        subject,
                        audience,
                        nonce,
                        issuance_time,
                        expiration_time,
                    };
                    let token = jsonwebtoken::encode(&jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256), &claims, &encoding_key)?;
                    Ok(RequestHandling::Respond(NcogResponse::IdentityVerificationToken {  token }))
                } else {
                    Ok(RequestHandling::Respond(NcogResponse::Error { message: Some("cannot request identity verification without being authenticated".to_string())}))
                }
            }
        }
    }

    async fn lookup_account_from_installation_id(
        &self,
        installation_id: Uuid,
    ) -> anyhow::Result<Option<Handle<Self::Account>>> {
        let account = ConnectedAccount::lookup(installation_id).await?;
        Ok(Some(Handle::new(account)))
    }

    fn protocol_version_requirements(&self) -> VersionReq {
        ncog_protocol_version_requirements()
    }

    async fn lookup_or_create_installation(
        &self,
        _client: &ConnectedClient<Self>,
        installation_id: Option<Uuid>,
    ) -> anyhow::Result<InstallationConfig> {
        let installation = database::lookup_or_create_installation(installation_id).await?;
        Ok(InstallationConfig::from_vec(
            installation.id,
            installation.private_key.unwrap(),
        )?)
    }

    async fn client_reconnected(
        &self,
        client: &ConnectedClient<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        if let Some(account) = client.account().await {
            let account = account.read().await;

            Ok(RequestHandling::Respond(NcogResponse::Authenticated {
                profile: account.profile.clone(),
                permissions: account.permissions.clone(),
            }))
        } else {
            Ok(RequestHandling::Respond(NcogResponse::Unauthenticated))
        }
    }

    async fn new_client_connected(
        &self,
        _client: &ConnectedClient<Self>,
    ) -> anyhow::Result<RequestHandling<Self::Response>> {
        Ok(RequestHandling::Respond(NcogResponse::Unauthenticated))
    }

    async fn account_associated(&self, client: &ConnectedClient<Self>) -> anyhow::Result<()> {
        if let Some(installation) = client.installation().await {
            if let Some(account) = client.account().await {
                let account_id = {
                    let account = account.read().await;
                    account.id()
                };
                database::set_installation_account_id(&pg(), installation.id, Some(account_id))
                    .await?;
                return Ok(());
            }
        }
        anyhow::bail!("account_associated called with either no installation or account")
    }

    async fn handle_websocket_error(&self, _err: warp::Error) -> ErrorHandling {
        println!("Error: {:?}", _err);
        ErrorHandling::Disconnect
    }

    async fn client_disconnected(&self, _client: &ConnectedClient<Self>) -> anyhow::Result<()> {
        Ok(())
    }
}

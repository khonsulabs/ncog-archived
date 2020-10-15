# Authentication with ncog

This document's goal is to describe the flow in more detail than most people need to be aware of.

## Game Client Login

To initiate a login, request a login URL with `NcogRequest::AuthenticationUrl(OAuthProvider::Twitch)`. The server will respond with `NcogResponse::AuthenticateAtUrl`, and will automatically try to open the browser for you. Upon successful login, the game client will receive a state change event with `AuthState::Authenticated`.

## Validating Ncog Identities

If a server needs to know if it can trust a client saying that it's a particular Ncog user, design a flow between the game client and your server such that these steps happen:

- The game server requests the current valid list of JSON Web Keys. This value should be cached.
- The client, upon logging into Ncog, asks the Game Server to initiate a login flow.
- The game server should respond with a one-time random value "nonce" and an `audience` value.
- The client sends a request `NcogRequest::RequestIdentityVerificationToken` with the values provided by the game server.
- Ncog will produce a JWT containing claims that include the nonce and audience, and send it in `NcogResponse::IdentityVerificationToken`
- The client sends this token to the game server
- The game server uses the JSON Web Keys to decode the JWT. The server should validate that the nonce and audience match the values it originally provided. If everything matches, you can assume the `subject` (ncog user id), `ncog_profile`, and `ncog_permissions` fields contain valid information that came from Ncog and nowhere else

## Want traditional OAuth/OpenID Connect?

For Khonsu Labs' vision of ease of use of the game client, this flow was designed for minimal friction. There is no need to redirect from the browser back to the game client, because the login success message is delivered over an already established websocket connection.

That being said, if anyone is using Ncog but wants a traditional OAuth/OpenID Connect flow, please file an issue!

## schema brainstorming

Accounts
id
screenname NULL UNIQUE

Installations
id uuid
account_id NULL

OauthTokens
id
account_id
service
refresh_token
access_token
expires

ItchioProfile
id
username
url
email
oauth_token_id

Roles
id
name
universe_id

PermissionStatement
id
role_id
resource: ncog
action
allow/deny

AccountRoles
account_id
role_id

Universes
id
name
parent_universe_id

UniverseGlobals
universe_id
name
value

Entities
id
state

Avatars
id
universe_id
account_id
name
entity_id

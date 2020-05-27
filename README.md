# What is ncog?

ncog is the engine behind [ncog.link](https://ncog.link). It is [open source under the MIT License](/LICENSE). For more information about the service, please see the [vision page on ncog.link](https://ncog.link/about/vision).

# Running the code yourself

## server

Requirements:

- Install [wasm-pack](https://github.com/rustwasm/wasm-pack): `cargo install wasm-pack`
- Install [rollup](https://rollupjs.org/): `npm install -g rollup`
- Install [sass](https://sass-lang.com/): `npm install -g sass`
- Create a PostgreSQL 11+ database with a user to access it:
  - `CREATE ROLE ncog_user LOGIN PASSWORD '...';`
  - `CREATE DATABASE ncog OWNER ncog_user;`
- Setup `.env` in the workspace root:
  ```
  DATABASE_URL=postgres://ncog_user:<PASSWORD>@host:port/ncog
  ITCHIO_CLIENT_ID=...
  ```

Building:

- `cd webapp`
- `sass sass/styles.sass static/styles.css`
- ``

## schema brainstorming

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

# ncog

ncog is the engine behind [ncog.id](https://ncog.id). It is [open source under the MIT License](/LICENSE). For more information about the service, please see the [vision page on ncog.id](https://ncog.id/about/vision).

ncog is written in [rust](https://rust-lang.org). The webserver is written using [warp](https://lib.rs/warp). The webapp is a [wasm](https://webassembly.org/) application built with [yew](https://yew.rs/). The game client is written built with [Kludgine](../kludgine), a custom game engine.

## Running the code yourself

### Initial Setup

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
- Run migrations: `cargo run --bin migrator`

### Building:

- `cargo make build` (add `-p release` to make release builds)

### Running

#### Webapp

There's no such thing as "running" this, but if you're actively developing it, you can start a watch command that the server will automatically serve (no auto-reloading):

`cd webapp && cargo make watch`

#### Server

- `cargo run --package server`

The webserver is running at `localhost:7878`

#### Client

- `cargo run --package client`

## Contributing

This project is in its infancy. If you want to contribute, please reach out to [@ecton](https://github.com/ecton) before attempting any major pull requests or minor ones that change existing functionality (without first determining if it's a bug or by design).

# Development Instructions

For now, this is just a quick checklist to remind me of dependencies that
aren't tracked by Cargo.

If you want more details to help you get started, feel free to open an issue
(or find me online somewhere)!

## What to install

1. [Rust stable](https://www.rust-lang.org/tools/install)
2. [Docker](https://www.rust-lang.org/tools/install) (and [`docker-compose`](https://docs.docker.com/compose/install/) if that didn't already come with it)
3. [`cargo-run-bin`](https://github.com/dustinblackman/cargo-run-bin?tab=readme-ov-file#install)
4. [`cargo-binstall`](https://crates.io/crates/cargo-binstall) (optional, but _highly_ recommended)

## First-time setup

1. Install everything above.
2. Clone this repo.
3. Copy `.env.template` to `.env`: `cp -iv .env.template .env`
4. Edit the `TODO` placeholders in `.env` to fit your environment.
5. Load `.env` into your shell. I recommend [direnv](https://direnv.net/) for convenience, but `source .env` works too.
6. Install all the dev tools: `cargo make deps`
7. Start the local development database (as a Postgres container): `cargo make services`
8. Apply database migrations: `cargo make db`
9. Start all the dev servers: `cargo make dev`
10. Go to `http://localhost:8080`

## Useful dev script reference

These are some commands I run every time I work in this repo. Most of them are
meta-dev-scripts (they spawn further scripts). If you're curious about what
they expand to, read [`Makefile.toml`](Makefile.toml).

### List all dev scripts (help)

```bash
cargo make
```

or

```bash
cargo make help
```

### Install all the tooling dependencies

```bash
cargo make deps
```

### Start the development environment

```bash
cargo make dev
```

### Run database migrations and sync ORM definitions

```bash
cargo make db
```

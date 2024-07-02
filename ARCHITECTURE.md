# Architecture

This describes the major components of the project, including frameworks that
are useful to know about when developing.

If you want more detail on something (or notice that something seems
out-of-date), feel free to open an issue (or find me online somewhere)!

## Components

The site consists of...

- A single-page app (SPA) called `web` (that should probably be split up: #33)
- A server called `server` that serves the SPA, a JSON API, and a bit of HTML
- A PostgreSQL database that acts as the main data store

The site is deployed on my personal [Fly.io] organization and reachable at
https://www.emptyblock.dev

[Fly.io]: https://fly.io

## Frameworks, libraries, etc.

An early, self-imposed, unreasonable constraint I placed on this project is to
have all of the code written in Rust. Some things can't be Rust (yet?), but
that didn't stop me from trying!

It's probably worth knowing these names to help navigate code:

- Frontend framework: [Yew] (Rust code compiled to Wasm, inspired by Elm)
- Frontend styling: [Tailwind]
- Frontend build: [Trunk]

- Backend framework: [Axum]
- Object-relational mapping (ORM): [SeaORM]

- Database: [PostgreSQL]

[Yew]: https://yew.rs/
[Tailwind]: https://tailwindcss.com/
[Trunk]: https://trunkrs.dev/
[Axum]: https://github.com/tokio-rs/axum
[SeaORM]: https://www.sea-ql.org/SeaORM/
[PostgreSQL]: https://www.postgresql.org/

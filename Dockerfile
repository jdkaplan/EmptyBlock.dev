###############################################################################
# Build the server
###############################################################################
FROM rust:1.76.0-slim as build

WORKDIR /usr/local/src/ebd

# Copy in just enough to make `cargo fetch` work.
RUN mkdir src && touch src/main.rs
COPY Cargo.toml Cargo.lock ./

RUN cargo fetch

COPY . .
RUN cargo build --release

###############################################################################
# Make the runnable image
###############################################################################
FROM debian:12.5-slim

COPY --from=build \
    /usr/local/src/ebd/target/release/ebd \
    /usr/local/bin/ebd

CMD [ "/usr/local/bin/ebd" ]

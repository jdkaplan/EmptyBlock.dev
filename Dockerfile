###############################################################################
# Build the web app
###############################################################################
FROM rust:1.76.0 as bundle

WORKDIR /usr/local/src/ebd

RUN rustup target add wasm32-unknown-unknown

ENV TRUNK_VERSION="v0.18.8"
ENV TRUNK_DOWNLOAD_URL="https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz"
ENV TRUNK_SHA256="4dacb8b49ccb9a82ef2e67ffa37c94c1079cc41ad3368e8e6c07beed516d8e64"

# TODO: When `ADD --checksum` is more widely supported, use that instead.
RUN curl \
    --location "${TRUNK_DOWNLOAD_URL}" \
    --output /tmp/trunk.tar.gz \
    && (cd /tmp && echo "${TRUNK_SHA256}  trunk.tar.gz" | sha256sum --check) \
    && (cd /usr/local/bin && tar -zxvf /tmp/trunk.tar.gz) \
    && chmod +x /usr/local/bin/trunk \
    && rm /tmp/trunk.tar.gz

# Copy in just enough to make `cargo fetch` work.
RUN mkdir -p web/src \
 && touch web/src/main.rs \
 && echo "workspace.resolver = '2'\nworkspace.members = ['web']" > Cargo.toml
COPY Cargo.lock ./
COPY web/Cargo.toml ./web/Cargo.toml

RUN cargo fetch --target wasm32-unknown-unknown

COPY web web
RUN (cd web && trunk build --release)

###############################################################################
# Build the server
###############################################################################
FROM rust:1.76.0 as build

WORKDIR /usr/local/src/ebd

# Copy in just enough to make `cargo fetch` work.
RUN mkdir -p server/src \
 && touch server/src/main.rs \
 && echo "workspace.resolver = '2'\nworkspace.members = ['server']" > Cargo.toml
COPY Cargo.lock ./
COPY server/Cargo.toml ./server/Cargo.toml

RUN cargo fetch

COPY server server
RUN cargo build --release --package server

###############################################################################
# Build some deploy tools
###############################################################################

FROM rust:1.76.0 as tools

ENV SQUILL_VERSION="v0.8.0"
ENV SQUILL_DOWNLOAD_URL="https://github.com/jdkaplan/squill/releases/download/${SQUILL_VERSION}/squill-x86_64-unknown-linux-gnu.tar.xz"
ENV SQUILL_SHA256="5050d2de2e69b565d95b57f49553fcf409ad323b5fe9f080e4e599fd91272d61"

# TODO: When `ADD --checksum` is more widely supported, use that instead.
RUN curl \
    --location "${SQUILL_DOWNLOAD_URL}" \
    --output /tmp/squill.tar.xz \
    && (cd /tmp && echo "${SQUILL_SHA256}  squill.tar.xz" | sha256sum --check) \
    && (mkdir /tmp/squill && cd /tmp/squill && tar -xvf /tmp/squill.tar.xz --strip-components 1) \
    && (cp /tmp/squill/squill /usr/local/bin/squill) \
    && chmod +x /usr/local/bin/squill \
    && rm /tmp/squill.tar.xz

###############################################################################
# Make the runnable image
###############################################################################
FROM debian:12.5-slim

COPY migrations /src/migrations

COPY --from=tools \
    /usr/local/bin/squill \
    /usr/local/bin/squill

COPY --from=bundle \
    /usr/local/src/ebd/web/dist \
    /app/dist

COPY --from=build \
    /usr/local/src/ebd/target/release/server \
    /app/bin/server

CMD [ "/app/bin/server", "/app/dist" ]

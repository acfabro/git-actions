ARG INSTALL_FLAGS=""

FROM rust:bullseye AS builder

#RUN apk add --no-cache build-base musl-dev openssl-dev pkgconf
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev

COPY Cargo.toml Cargo.lock /usr/src/git-actions/
COPY src/ /usr/src/git-actions/src
COPY target/ /usr/src/git-actions/target

WORKDIR /usr/src/git-actions

ARG INSTALL_FLAGS
RUN cargo install --path . ${INSTALL_FLAGS}

FROM rust:1-slim-bullseye

WORKDIR /usr/local/bin
COPY --from=builder /usr/local/cargo/bin/git-actions ./
RUN chmod +x git-actions

ENTRYPOINT ["git-actions"]

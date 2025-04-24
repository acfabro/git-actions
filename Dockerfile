# Use the official Rust image as the base image for building the application
FROM rust:1.86.0-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf perl make

ENV SYSROOT=/dummy

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy only the Cargo.toml and Cargo.lock files first
COPY Cargo.toml Cargo.lock ./

# Copy the rest of the source code and other files
COPY src ./src

# Build the application
RUN cargo build --bins --release

# Use a minimal base image for the final stage
FROM scratch

ARG version=unknown
ARG release=unreleased

LABEL name="git-actions" \
    vendor="Rakuten Inc." \
    version=${version} \
    release=${release} \
    summary="Configurable actions based on Git events" \
    description="A Rust-based automation tool that listens for Git events and executes configurable actions based on customizable rules."

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/src/app/target/release/git-actions /

# Expose the port the application runs on
EXPOSE 8000

CMD ["./git-actions"]
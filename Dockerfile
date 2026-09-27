# syntax=docker/dockerfile:1

# rust:alpine already targets musl, so no cross-compile setup is needed
FROM rust:1.98-alpine3.24 AS builder

# musl-dev provides the crt objects the linker needs for a static binary
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY directory.html ./
COPY src ./src

# cache mounts keep the registry and build artifacts between builds;
# the binary is copied out because a mount is not part of the image layer
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/basic-http-rs /basic-http-rs

# the binary is fully static, so the image needs nothing but the binary itself
FROM scratch

COPY --from=builder /basic-http-rs /basic-http-rs

# no /etc/passwd here, so the user has to be numeric (65534 = nobody)
USER 65534:65534
WORKDIR /srv
EXPOSE 80

# serve path defaults to ".", so mount the site at /srv
ENTRYPOINT ["/basic-http-rs", "-p", "80"]

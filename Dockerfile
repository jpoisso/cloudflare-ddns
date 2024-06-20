##### Builder
FROM rust:alpine3.18 as builder

# Create fresh workspace
WORKDIR /usr/src
RUN apk add --no-cache musl-dev
RUN USER=root cargo new cloudflare-ddns

# Cache dependencies by themselves
COPY Cargo.toml Cargo.lock /usr/src/cloudflare-ddns/
WORKDIR /usr/src/cloudflare-ddns
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

# Build the application
COPY src /usr/src/cloudflare-ddns/src/
RUN touch /usr/src/cloudflare-ddns/src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

##### Runtime
FROM alpine:3.20.1 AS runtime

# Copy generated binary in runtime
COPY --from=builder /usr/src/cloudflare-ddns/target/x86_64-unknown-linux-musl/release/cloudflare-ddns /usr/local/bin
RUN chmod +x /usr/local/bin/cloudflare-ddns

# Run as regular user (not root)
RUN addgroup -S appgroup && adduser -S appuser -u 10001 -G appgroup
USER 10001

CMD ["/usr/local/bin/cloudflare-ddns"]
# Step 1: Build the Rust application
FROM rust:1 AS build

# libgit2-sys builds bundled libgit2 (needs cmake); git2 https needs openssl.
RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release

# Step 2: Minimal runtime image
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/mindmirror /usr/local/bin/mindmirror
EXPOSE 8080
ENTRYPOINT ["mindmirror"]
CMD ["serve"]

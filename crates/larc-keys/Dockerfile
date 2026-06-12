FROM rust:1.87-slim AS builder
WORKDIR /build

# Install build deps for rusqlite (bundled sqlite needs cc)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy source
COPY crates/larc-keys/Cargo.toml crates/larc-keys/
COPY crates/larc-keys/src        crates/larc-keys/src/

# Synthesise a minimal workspace root (avoids broken sibling crates)
RUN cat > Cargo.toml <<'TOML'
[workspace]
resolver = "2"
members = ["crates/larc-keys"]

[workspace.package]
version      = "0.1.0"
edition      = "2024"
license      = "Apache-2.0"
repository   = "https://github.com/TheLightArchitects/larc-crates"
rust-version = "1.87"

[workspace.dependencies]
axum               = { version = "0.8", features = ["macros"] }
tower-http         = { version = "0.6", features = ["trace"] }
tokio              = { version = "1", features = ["full"] }
rusqlite           = { version = "0.32", features = ["bundled"] }
serde              = { version = "1", features = ["derive"] }
serde_json         = "1"
thiserror          = "2"
tracing            = "0.1"
uuid               = { version = "1", features = ["v4", "serde"] }
chrono             = { version = "0.4", features = ["serde"] }
hmac               = "0.12"
sha2               = "0.10"
rand               = "0.8"
TOML

RUN cargo build --release -p larc-keys

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/larc-keys /usr/local/bin/larc-keys
RUN mkdir -p /data
VOLUME ["/data"]
EXPOSE 3800
CMD ["larc-keys"]

FROM rust:1.87-slim AS builder
WORKDIR /build

# Workspace skeleton for caching deps
COPY Cargo.toml Cargo.lock* ./
COPY crates/larc-keys/Cargo.toml crates/larc-keys/
# Stub out other workspace members so they don't block the build
COPY crates/larc-proxy/Cargo.toml crates/larc-proxy/
RUN mkdir -p crates/larc-keys/src crates/larc-proxy/src && \
    echo 'fn main(){}' > crates/larc-keys/src/main.rs && \
    echo 'fn main(){}' > crates/larc-proxy/src/main.rs && \
    cargo build --release -p larc-keys 2>/dev/null || true

# Real build
COPY crates/larc-keys/src crates/larc-keys/src/
RUN touch crates/larc-keys/src/main.rs && \
    cargo build --release -p larc-keys

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/larc-keys /usr/local/bin/larc-keys

RUN mkdir -p /data
VOLUME ["/data"]

EXPOSE 3800
CMD ["larc-keys"]

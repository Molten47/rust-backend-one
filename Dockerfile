# ── Build stage ──────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first — Docker layer caches dependencies
# so cargo build only re-runs when Cargo.toml/lock changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy full source and build for real
COPY . .
RUN touch src/main.rs
RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-auth-api .
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/.sqlx ./.sqlx

ENV SQLX_OFFLINE=true

EXPOSE 3000

CMD ["./rust-auth-api"]

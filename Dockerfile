FROM rust:slim-bookworm AS builder

WORKDIR /app

# Cache dependencies - copy only manifests first
COPY Cargo.toml Cargo.lock* ./
COPY apps/server/Cargo.toml apps/server/Cargo.toml

# Create dummy main to build deps
RUN mkdir -p apps/server/src && echo "fn main() {}" > apps/server/src/main.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release --manifest-path apps/server/Cargo.toml 2>/dev/null || true

# Now copy real source
COPY apps/server apps/server
# Touch main.rs so cargo knows to rebuild it
RUN touch apps/server/src/main.rs
RUN cargo build --release --manifest-path apps/server/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/server /usr/local/bin/server
ENV SERVER_PORT=8080
CMD ["server"]

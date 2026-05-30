# syntax=docker/dockerfile:1

FROM node:22-bookworm AS web
WORKDIR /web
COPY web/package.json web/package-lock.json* ./
RUN npm install
COPY web/ ./
ENV VITE_API_BASE=
RUN npm run build

FROM rust:1.85-bookworm AS rust
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
RUN cargo build --release -p orchestrator-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust /app/target/release/orchestrator-server /usr/local/bin/
COPY --from=web /web/dist /app/web/dist

ENV ORCHESTRATOR_PROFILE=prod \
    ORCHESTRATOR_BIND_ADDR=0.0.0.0 \
    ORCHESTRATOR_PORT=47821 \
    ORCHESTRATOR_DATA_DIR=/data \
    ORCHESTRATOR_STATIC_DIR=/app/web/dist

VOLUME ["/data", "/workspace"]
EXPOSE 47821

ENTRYPOINT ["orchestrator-server", "serve"]

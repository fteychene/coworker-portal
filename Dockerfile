# ─── Stage 1: Frontend ────────────────────────────────────────────────────────
FROM node:22-alpine AS frontend

WORKDIR /app/frontend

# Install dependencies first (layer-cached until package*.json changes)
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

# Build
COPY frontend/ ./
RUN npm run build


# ─── Stage 2: Rust build ──────────────────────────────────────────────────────
FROM rust:1.88-slim-bookworm AS builder

# System deps for sqlx (native TLS not needed — we use rustls)
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests and pre-fetch dependencies (layer-cached until Cargo.* changes)
COPY Cargo.toml Cargo.lock ./
# Dummy main so cargo can resolve the dependency graph
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    cargo fetch && \
    rm -rf src

# Copy the full source
COPY . .

# Bring in the pre-built frontend dist so build.rs skips npm
COPY --from=frontend /app/frontend/dist ./frontend/dist
ENV SKIP_FRONTEND_BUILD=1

RUN cargo build --release


# ─── Stage 3: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/coworking-tooling ./
COPY --from=builder /app/public ./public

EXPOSE 3000

CMD ["./coworking-tooling"]

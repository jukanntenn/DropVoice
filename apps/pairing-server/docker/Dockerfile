# syntax=docker/dockerfile:1
# DropVoice pairing-server multi-stage Dockerfile (all-in-one image).
#
# Design (spec 11 §12, mirrors markpost docker/Dockerfile):
#   - rust:alpine builder via cargo-chef; sqlx's `sqlite` feature enables
#     bundled libsqlite3 (statically linked), so the runtime image needs no
#     system sqlite.
#   - cargo-chef caches the dependency build layer (rebuilt only when
#     Cargo.toml changes), keeping iterative rebuilds fast.
#   - `apps/desktop/src-tauri` is a full Tauri 2 project that won't build on
#     musl, so it's stubbed out during dependency caching and excluded from the
#     app build.
#   - Runtime image bundles s6-overlay + Caddy + Rust binary (all-in-one).
#
# Build context: repository root (../../ from docker/), to access the workspace
# Cargo.toml + Cargo.lock + apps/pairing-server.
#
# VERSION is the version string baked into the image (ENV APP_VERSION + image
# label). Passed by build.py (git describe) or CI (git tag); defaults to "dev"
# for bare `docker build`.

# ===========================================================================
# Stage 1: Chef — prepare the dependency recipe
# ===========================================================================
FROM rust:alpine AS chef
RUN apk add --no-cache musl-dev make perl curl
RUN cargo install cargo-chef --locked
# crates.io 下载加固：弱网（SSL_ERROR_SYSCALL 抖动）下重试 10 次、超时 300s。
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_TIMEOUT=300 \
    CARGO_HTTP_LOW_SPEED_LIMIT=10
WORKDIR /app

# ===========================================================================
# Stage 2: Planner — fingerprint which deps need building
# ===========================================================================
FROM chef AS planner
# Copy only manifests + lockfile so the recipe is source-independent.
COPY Cargo.toml Cargo.lock ./
COPY apps/pairing-server/Cargo.toml apps/pairing-server/Cargo.toml
COPY apps/pairing-server/migrations apps/pairing-server/migrations
# Stub out desktop/src-tauri before `cargo chef prepare`: cargo-chef runs
# `cargo metadata`, which must resolve the entire workspace, but the real
# Tauri project (webkit/gtk deps) won't build on musl. The stub makes the
# member resolvable without its real sources (which aren't COPYed here).
RUN mkdir -p apps/desktop/src-tauri/src && \
    printf '[package]\nname = "dropvoice-desktop"\nversion = "0.2.0"\nedition = "2021"\n\n[lib]\nname = "dropvoice_desktop_lib"\npath = "src/lib.rs"\n' > apps/desktop/src-tauri/Cargo.toml && \
    printf '' > apps/desktop/src-tauri/src/lib.rs
RUN cargo chef prepare --recipe-path recipe.json

# ===========================================================================
# Stage 3: Builder — compile deps (cached) then the app
# ===========================================================================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Stub out desktop/src-tauri (see planner).
RUN mkdir -p apps/desktop/src-tauri/src && \
    printf '[package]\nname = "dropvoice-desktop"\nversion = "0.2.0"\nedition = "2021"\n\n[lib]\nname = "dropvoice_desktop_lib"\npath = "src/lib.rs"\n' > apps/desktop/src-tauri/Cargo.toml && \
    printf '' > apps/desktop/src-tauri/src/lib.rs

# Build dependencies first — this layer is cached unless Cargo.toml changes.
RUN cargo chef cook --release --recipe-path recipe.json

# Copy real pairing-server sources.
COPY apps/pairing-server/Cargo.toml apps/pairing-server/Cargo.toml
COPY apps/pairing-server/migrations apps/pairing-server/migrations
COPY apps/pairing-server/src apps/pairing-server/src

RUN cargo build --release -p dropvoice-pairing-server && \
    cp target/release/pairing-server /usr/local/bin/pairing-server

# ===========================================================================
# Stage 3.5: Frontend builder — build mobile PWA + landing with pnpm
# ===========================================================================
# Both frontends are baked into the image so the image is the single,
# self-contained artifact: deployments upload nothing, and frontend versions
# can never drift from the backend version (deployments used to upload
# apps/mobile/dist from the operator's working tree, which could be stale).
# Layout at runtime: /app/mobile-dist (app.<domain>) + /app/landing-dist
# (apex <domain>) — see docker/Caddyfile / ansible Caddyfile.j2.
FROM node:22-alpine AS frontend-builder
RUN npm install -g pnpm@11.0.0 --no-fund --no-audit
WORKDIR /app
# Manifests first so `pnpm install` is cached independently of sources.
COPY pnpm-workspace.yaml package.json pnpm-lock.yaml .npmrc tsconfig.base.json ./
COPY apps/mobile/package.json apps/mobile/package.json
COPY apps/landing/package.json apps/landing/package.json
COPY apps/desktop/package.json apps/desktop/package.json
COPY packages/core/package.json packages/core/package.json
COPY packages/i18n/package.json packages/i18n/package.json
COPY packages/ui/package.json packages/ui/package.json
RUN pnpm install --frozen-lockfile
COPY apps/mobile apps/mobile
COPY apps/landing apps/landing
COPY packages/core packages/core
COPY packages/i18n packages/i18n
COPY packages/ui packages/ui
RUN pnpm --filter @dropvoice/mobile build && \
    pnpm --filter @dropvoice/landing build

# ===========================================================================
# Stage 4: Runtime — alpine + s6-overlay + Caddy
# ===========================================================================
FROM alpine:3.21

ARG TARGETARCH
ARG S6_OVERLAY_VERSION=3.2.0.2
ARG CADDY_VERSION=v2.10.2
ARG VERSION=dev

RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    curl

# s6-overlay process supervisor
ADD "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz" /tmp/s6-overlay-noarch.tar.xz
RUN S6_ARCH=$(case "${TARGETARCH}" in amd64) echo "x86_64" ;; arm64) echo "aarch64" ;; esac) && \
    curl -fsSL "https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-${S6_ARCH}.tar.xz" -o /tmp/s6-overlay-arch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-arch.tar.xz && \
    rm -f /tmp/s6-overlay-*.tar.xz

# Caddy reverse proxy (the only published port; backend stays loopback-only)
RUN curl -fsSL \
    "https://github.com/caddyserver/caddy/releases/download/${CADDY_VERSION}/caddy_${CADDY_VERSION##v}_linux_${TARGETARCH}.tar.gz" \
    | tar xz -C /usr/local/bin caddy && \
    chmod +x /usr/local/bin/caddy

# s6 service definitions
COPY apps/pairing-server/docker/s6/cont-init.d /etc/cont-init.d
COPY apps/pairing-server/docker/s6/s6-rc.d /etc/s6-overlay/s6-rc.d
RUN chmod +x /etc/cont-init.d/*.sh

# Caddy config — image-baked default (HTTP self-hosted form). Overridden at
# runtime by a mounted volume (acceptance: Caddyfile.local, deployed envs:
# Ansible-rendered Caddyfile), all mounted over /etc/caddy/Caddyfile.
COPY apps/pairing-server/docker/Caddyfile /etc/caddy/Caddyfile

# Rust binary (libsqlite3 is statically linked via sqlx's bundled sqlite — no .so needed).
COPY --from=builder /usr/local/bin/pairing-server /app/pairing-server

# Mobile PWA + landing (built in the frontend-builder stage) — served by Caddy
# at /app/mobile-dist (app host) + /app/landing-dist (apex host).
COPY --from=frontend-builder /app/apps/mobile/dist /app/mobile-dist
COPY --from=frontend-builder /app/apps/landing/dist /app/landing-dist

# Migrations are embedded in the binary via sqlx::migrate!, but keep a copy for
# manual inspection / recovery.
COPY --from=builder /app/apps/pairing-server/migrations /app/migrations

# Data + log directories
RUN mkdir -p /app/data /app/log

ENV RUST_LOG=info
ENV S6_KEEP_ENV=1
ENV APP_VERSION=${VERSION}
LABEL org.opencontainers.image.version=${VERSION}

WORKDIR /app
ENTRYPOINT ["/init"]

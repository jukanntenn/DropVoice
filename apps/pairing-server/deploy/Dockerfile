# 配对服务器 Dockerfile（多阶段构建）。
# docker-compose.test.yml 与未来生产部署共用。

FROM rust:1.97-slim AS builder
WORKDIR /build
# 利用 docker 缓存：先拷依赖清单
COPY Cargo.toml Cargo.lock* ./
COPY apps/pairing-server/Cargo.toml ./apps/pairing-server/Cargo.toml
COPY apps/pairing-server/migrations ./apps/pairing-server/migrations
COPY apps/pairing-server/src ./apps/pairing-server/src
# 仅构建配对服务器二进制
RUN cargo build --release -p dropvoice-pairing-server && \
    cp target/release/pairing-server /usr/local/bin/pairing-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/pairing-server /usr/local/bin/pairing-server
EXPOSE 38424
ENV LISTEN_ADDR="0.0.0.0:38424"
ENV RUST_LOG="info"
CMD ["pairing-server"]

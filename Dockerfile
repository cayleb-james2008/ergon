# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /src

# Copy manifests and source (committed assets/ + catalog/ embed directly at compile time).
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY assets ./assets
COPY catalog ./catalog

RUN cargo build --release --locked --bin ergon && strip /src/target/release/ergon

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -g 10001 ergon && \
    useradd -u 10001 -g 10001 -m -d /home/ergon -s /usr/sbin/nologin ergon && \
    mkdir -p /data && \
    chown -R 10001:10001 /data

COPY --from=builder /src/target/release/ergon /usr/local/bin/ergon

ENV ERGON_HOST=0.0.0.0 \
    ERGON_PORT=8787 \
    ERGON_DATA_DIR=/data \
    ERGON_NO_OPEN=1 \
    ERGON_TRUST_PROXY=1 \
    ERGON_DISABLE_HOST_APP_SCAN=1 \
    ERGON_MODEL_BASE_URL=https://opencode.ai/zen/go/v1 \
    ERGON_MODEL_ID=deepseek-v4.1-flash

VOLUME ["/data"]
EXPOSE 8787

USER 10001:10001

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["ergon", "healthcheck"]

ENTRYPOINT ["ergon"]

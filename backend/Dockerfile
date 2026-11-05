FROM rust:1.99.1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --no-install-recommends --yes ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --uid 10001 lukas

COPY --from=builder /app/target/release/lukas-api /usr/local/bin/lukas-api

USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
    CMD ["curl", "--fail", "--silent", "http://127.0.0.1:8080/health/ready"]
ENTRYPOINT ["/usr/local/bin/lukas-api"]

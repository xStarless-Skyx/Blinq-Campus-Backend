# Build Stage
FROM rust:1.92.0-slim-bookworm AS builder
USER 0:0
WORKDIR /home/rust/src

ARG SERVICE=api

# Human-readable service names mapped to workspace binaries:
# api -> revolt-delta
# events -> revolt-bonfire
# files -> revolt-autumn
# metadata -> revolt-january
# gifbox -> revolt-gifbox
# crond -> revolt-crond
# pushd -> revolt-pushd
# voice-ingress -> revolt-voice-ingress

ENV CARGO_BUILD_JOBS=1 \
    CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_RELEASE_LTO=false \
    RUSTFLAGS="-C debuginfo=0"

# Install standard build requirements
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev \
    gcc && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build only the selected service to keep Docker memory usage manageable.
RUN case "${SERVICE}" in \
        api) APP="revolt-delta" ;; \
        events) APP="revolt-bonfire" ;; \
        files) APP="revolt-autumn" ;; \
        metadata) APP="revolt-january" ;; \
        gifbox) APP="revolt-gifbox" ;; \
        crond) APP="revolt-crond" ;; \
        pushd) APP="revolt-pushd" ;; \
        voice-ingress) APP="revolt-voice-ingress" ;; \
        *) echo "Unsupported SERVICE: ${SERVICE}" >&2; exit 1 ;; \
    esac && \
    cargo build --locked --release --package "${APP}" --bin "${APP}" && \
    cp "target/release/${APP}" "/usr/local/bin/service-bin"

# Final Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/service-bin /usr/local/bin/service-bin

CMD ["/usr/local/bin/service-bin"]

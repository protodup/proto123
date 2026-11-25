# Multi-stage static build for Proto123 (research R13, FR-041).
FROM rust:1.99-slim AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl tar \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown

ARG TRUNK_VERSION=0.22.2
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) target="x86_64-unknown-linux-gnu" ;; \
      arm64) target="aarch64-unknown-linux-gnu" ;; \
      *) echo "unsupported arch: $arch" >&2; exit 1 ;; \
    esac; \
    asset="trunk-${target}.tar.gz"; \
    url="https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/${asset}"; \
    curl -fsSL "$url" -o "/tmp/${asset}"; \
    curl -fsSL "${url}.sha256" -o "/tmp/${asset}.sha256"; \
    echo "$(awk '{print $1}' "/tmp/${asset}.sha256")  /tmp/${asset}" | sha256sum -c -; \
    tar -xzf "/tmp/${asset}" -C /usr/local/bin trunk; \
    rm -f "/tmp/${asset}" "/tmp/${asset}.sha256"

ARG BINARYEN_VERSION=version_131
RUN set -eux; \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
      amd64) target="x86_64-linux" ;; \
      arm64) target="aarch64-linux" ;; \
      *) echo "unsupported arch: $arch" >&2; exit 1 ;; \
    esac; \
    asset="binaryen-${BINARYEN_VERSION}-${target}.tar.gz"; \
    url="https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}/${asset}"; \
    curl -fsSL "$url" -o "/tmp/${asset}"; \
    curl -fsSL "${url}.sha256" -o "/tmp/${asset}.sha256"; \
    echo "$(awk '{print $1}' "/tmp/${asset}.sha256")  /tmp/${asset}" | sha256sum -c -; \
    mkdir -p "/root/.cache/trunk/wasm-opt-${BINARYEN_VERSION}"; \
    tar -xzf "/tmp/${asset}" --strip-components=1 -C "/root/.cache/trunk/wasm-opt-${BINARYEN_VERSION}"; \
    rm -f "/tmp/${asset}" "/tmp/${asset}.sha256"

WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml Trunk.toml ./
COPY .cargo ./.cargo
COPY crates ./crates
COPY apps ./apps
COPY web ./web
COPY xtask ./xtask

RUN trunk build --release

FROM nginx:alpine AS runtime
COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80

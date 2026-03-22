# ── base ────────────────────────────────────────────────────────────────────
FROM rust:1.94-slim-bookworm AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    pkg-config \
    libssl-dev \
    git \
    curl \
 && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl \
 && rustup component add rustfmt clippy

RUN groupadd -g 1000 rustdev && useradd -u 1000 -g rustdev -m rustdev

WORKDIR /workspace

# ── dev ─────────────────────────────────────────────────────────────────────
FROM base AS dev

RUN cargo install cargo-watch --locked && cargo install cargo-edit --locked \
 && chown -R rustdev:rustdev /usr/local/cargo

USER rustdev

CMD ["bash"]

# ── release-builder ─────────────────────────────────────────────────────────
FROM base AS release-builder

# Cache dependencies before copying real source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --target x86_64-unknown-linux-musl \
 && rm -rf src

# Build for real
COPY src ./src
RUN touch src/main.rs \
 && cargo build --release --target x86_64-unknown-linux-musl

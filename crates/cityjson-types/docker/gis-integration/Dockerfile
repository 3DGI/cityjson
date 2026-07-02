FROM rust:1.93-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        clang \
        libclang-dev \
        libgeos-dev \
        pkg-config \
        postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/crates/cityjson-types/tools/gis-integration

# Use the official Rust image to build the application
FROM rust:latest AS builder
WORKDIR /usr/src/spacejam
COPY . .
RUN  --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,sharing=private,target=/src/target \
    apt-get update && \
    apt-get install -y protobuf-compiler libclang-dev && \
    cargo build -p spacejam --release --features rocksdb
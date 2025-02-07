# Use the official Rust image to build the application
FROM rust:latest AS builder
WORKDIR /usr/src/spacejam
COPY . .
RUN  --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
    --mount=target=/var/cache/apt,type=cache,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,sharing=private,target=target \
    apt-get update && \
    apt-get install -y protobuf-compiler libclang-dev && \
    cargo build -p spacejam --release && \
    cp target/release/spacejam /usr/local/bin/spacejam

# Use a smaller image for the final output
FROM debian:bookworm-slim
COPY --from=builder /usr/local/bin/spacejam /usr/local/bin/spacejam
ENTRYPOINT ["spacejam"]

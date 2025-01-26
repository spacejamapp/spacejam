# Use the official Rust image to build the application
#
# see also docker/builder.dockerfile
FROM rust:latest AS builder
WORKDIR /usr/src/spacejam
COPY . .
RUN  --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,sharing=private,target=/src/target \
    apt-get update && \
    apt-get install -y protobuf-compiler libclang-dev && \
    cargo build -p spacejam --release --features rocksdb

# Use a smaller image for the final output
#
# copy the binary from the builder stage
FROM debian:bullseye-slim
COPY --from=builder /lib/aarch64-linux-gnu/libc.so.1 /lib/aarch64-linux-gnu/libc.so.1
COPY --from=builder /usr/lib/aarch64-linux-gnu/libstdc++.so.6 /usr/lib/aarch64-linux-gnu/libstdc++.so.6
COPY --from=builder /usr/src/spacejam/target/release/spacejam /usr/local/bin/spacejam
CMD ["spacejam"]

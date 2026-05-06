# Spacejam image (jam-conformance fuzz target compatible).
#
# When run with JAM_FUZZ set, the binary listens on JAM_FUZZ_SOCK_PATH for the
# fuzzer protocol (fuzz-proto/README.md). Without JAM_FUZZ, it behaves as the
# normal CLI (run, key, fuzz subcommands).
#
# Required env vars in fuzz mode:
#   JAM_FUZZ            Enable fuzz target mode (any value).
#   JAM_FUZZ_SPEC       Protocol parameters: tiny | full (only tiny supported).
#   JAM_FUZZ_DATA_PATH  Directory for target data persistence.
#   JAM_FUZZ_SOCK_PATH  Unix domain socket path for fuzzer communication.
#   JAM_FUZZ_LOG_LEVEL  Optional. error | warn | info | debug | trace.
#
# Build the amd64 binary locally first:
#   cargo build -p spacejam --release --target x86_64-unknown-linux-gnu
#   cp target/x86_64-unknown-linux-gnu/release/spacejam target/release/spacejam
# then `docker build -f docker/spacejam.Dockerfile -t spacejam:fuzz .`

FROM --platform=linux/amd64 debian:bookworm-slim
COPY target/release/spacejam /usr/local/bin/spacejam
ENTRYPOINT ["spacejam"]

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
# Build via `make docker` (depends on `make linux-amd64`).

FROM debian:bookworm-slim
COPY target/x86_64-unknown-linux-gnu/prod/spacejam /usr/local/bin/spacejam
ENTRYPOINT ["spacejam"]

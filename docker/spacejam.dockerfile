# Spacejam image (jam-conformance fuzz target compatible).
#
# Ships both tiny and full binaries. JAM_FUZZ_SPEC selects which one runs:
#   JAM_FUZZ_SPEC=full  → /usr/local/bin/spacejam-full
#   JAM_FUZZ_SPEC=tiny  → /usr/local/bin/spacejam-tiny  (default)
#
# Required env vars in fuzz mode:
#   JAM_FUZZ            Enable fuzz target mode (any value).
#   JAM_FUZZ_SPEC       Protocol parameters: tiny | full.
#   JAM_FUZZ_DATA_PATH  Directory for target data persistence.
#   JAM_FUZZ_SOCK_PATH  Unix domain socket path for fuzzer communication.
#   JAM_FUZZ_LOG_LEVEL  Optional. error | warn | info | debug | trace.
#
# Build args:
#   SPACEJAM_INTERP     Set to 1 to bake an interpreter-only image (used by
#                       `make fuzz` for AOT-vs-interpreter A/B on NUMA hosts).
#
# Build via `make docker` (regular) or `make fuzz` (regular + interpreter).

FROM debian:bookworm-slim
ARG SPACEJAM_INTERP=""
COPY target/x86_64-unknown-linux-gnu/prod/spacejam-tiny /usr/local/bin/spacejam-tiny
COPY target/x86_64-unknown-linux-gnu/prod/spacejam-full /usr/local/bin/spacejam-full
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
ENV SPACEJAM_INTERP=${SPACEJAM_INTERP}
ENTRYPOINT ["entrypoint.sh"]

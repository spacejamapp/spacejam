FROM debian:bookworm-slim
COPY docker/polkajam/polkajam /usr/local/bin/polkajam
ENTRYPOINT ["polkajam"]

# Network

The network implementation follows [JAM-NP](https://github.com/zdave-parity/jam-np/blob/main/simple.md), here in this section we summarize the protocol as implementation docs.

## Encryption and handshake

- `QUIC` with `TLS 1.3` for the connection encryption and peer authentication.
- Both the client and the server must present `X.509` certificates.
  - `ed25519` as signature algorithm.
  - if the peer is a validator, this `ed25519` key must be published on chain.
  - base-32 encoded using the aplhabet `abcdefghijklmnopqrstuvwxyz234567`

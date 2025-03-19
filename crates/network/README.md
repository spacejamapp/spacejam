## TODOs

- [ ] handle blocks(not the first) from the next epoch
- [ ] on receiving invalid block (unable to validate but not invalid), request the ancestors
      of the block and try to validate them, this may require an intent implementation for the block requester.
- [ ] stream decoder
- [ ] embed the peer info into the validator metadata.

## SpaceJam Network

QUIC based JAM network protocol implementation.

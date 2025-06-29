## TODOs

- [ ] handle blocks(not the first) from the next epoch
- [ ] on receiving invalid block (unable to validate but not invalid), request the ancestors
      of the block and try to validate them, this may require an intent implementation for the block requester.
- [ ] handle fork chains
- [ ] handle incorrect order of blocks, e.g. block#1000 is received before block#999.

## SpaceJam Network

QUIC based JAM network protocol implementation.

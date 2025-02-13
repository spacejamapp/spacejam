# Milestones

Milestones following the [JAM announcement](https://jam.web3.foundation/).

## 1. Block Importer

- TODOs
  - [ ] the calculation of rotation period seems not correct in `reports`.
- State Transaction Functions

  - [x] Section 6 - [Block Production and Chain Growth](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/safrole)
  - [x] Section 7 - [Recent Blocks History](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/history)
  - [x] Section 8 - Authorization
  - [x] Section 10 - [Disputes, Verdicts and Judgements](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/disputes)
  - [x] Section 11 - [Reporting](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/reports) and [Assurances](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/assurances)
  - [ ] Section 12 - Accumulation
  - [x] Section 13 - [Activity Statistics](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/statistics)

- Others
  - [x] Appendix A - [Polkadot Virtual Machine](https://github.com/w3f/jamtestvectors/pull/3)
  - [x] Appendix C - [Jam Codec](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/codec)
  - [x] Appendix E.1 - [Binary Merkle Trees](https://github.com/clearloop/jam-test-vectors/tree/polkajam-vectors/trie)
  - [x] Appendix F - [Fishter-Yates Shuffle](https://github.com/w3f/jamtestvectors/pull/17)
  - [x] Appendix H - [Erasure Coding](https://github.com/w3f/jamtestvectors/pull/4)

## 2. Block Authoring

Block authoring in SpaceJam consists of several key components:

- [x] Launch tiny network (6 validators) with docker compose
- [x] Mock the full network ( assign to Alice, author empty block, empty accumulation, empty transactions, import blocks from network )
- [ ] Transaction pool management
- [x] Validator selection (mb require sort of auto send tx?)
- [x] Block authoring service
- [ ] Network propagation (jam-np)
- [x] Block sealing with VRF

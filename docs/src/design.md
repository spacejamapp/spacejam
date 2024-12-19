# Design

As a robust and efficient Jam Client, our system runs with the following principles:

- Limited database IO for block import.
  - Only read for once for each block.
  - Only write for once for each block.
- Asynchronous validation of extrinsics.
- AOT execution of programs (contracts, services, etc.)

## Block Validation

Before importing a block into the blockchain, it must be validated by the following components
to be finalized. (currently just modules in spacejam)

- Header
- Extrinsics (Body)
  - assurance
  - dispute
  - guarantee
  - tickets
  - preimage
- History

### Transaction Pool

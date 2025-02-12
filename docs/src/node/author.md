# Author

The **block authoring** service is triggered on each new time slot, if the current validator is elected,
say, the bandersnatch public key is included in the **sealing series of safrole**, it will:

- Select the **valid extrinsics** to include in the block.
  - Drop outdated extrinsics from the memory pool.
- Build the block from the latest block on the **longest chain**.
- Seal the block with VRF.
- Submit the block to the network.

## 1. Extrinsic Pool

The table below shows the size limits of the extrinsics in a single block, it could be scaled to N
blocks due to the configuration of the extrinsic pool.

| Name       | Limits              | graypaper |
| ---------- | ------------------- | --------- |
| Tickets    | K (16)              | true      |
| Guarantees | 2 MB ( 42 \* 48KB ) | false     |
| Disputes   | E (600)             | false     |
| Preimages  | -                   | false     |
| Assurances | C (341)             | true      |

We currently only have a memory pool for the extrinsics, each extrinsic should pass the basic
validation, eg. try executing it, and check the validity of the state changes before being
included in the extrinsic pool.

### 1.1. Tickets

The maximum number of tickets can be submitted in a single extrinsic is 16(`K = 16`).

- Once the sealing series is fitted to the EPOCH_LENGTH (`E = 600`), the pool will not accepting new
  ticket extrinsics anymore till the next epoch.
- Once the time slots have exceeded the ticket submission period (`Y = 500`), the pool will not accepting new
  ticket extrinsics anymore till the next epoch.
- All left tickets in the pool will be immediately dropped right after the two situations above, and start
  to accept new ticket extrinsics again.

### 1.2. Guarantees

No explicit limits on the number of guarantees, in spacejam, we temporarily limit it within `1024`,
and it will be lifted in the future.

### 1.3. Disputes

### 1.4. Preimages

### 1.5. Assurances

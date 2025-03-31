# Extrinsic

There are 5 types of extrinsics in the runtime:

| Extrinsic | Description                            |
| --------- | -------------------------------------- |
| Assurance | Assure the availability of a report    |
| Dispute   | Dispute the availability of a report   |
| Guarantee | Guarantee the availability of a report |
| Preimage  | Request a preimage for a report        |
| Ticket    | Request a ticket for a report          |

On state transition, we are using these `extrinsics` from the new block and the current `state`
on chain as the **input**, yields the new `state` as the **output** for the new block, which will
be stored in our database as the `state` snapshot of the block.

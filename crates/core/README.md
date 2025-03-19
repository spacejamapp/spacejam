## TODOs

- [ ] record the latest finalized head for restarting the node
- [ ] introduce storage for states
- [ ] introduce storage for grandpa, maybe we can keep it in memory, however, we need
      that kind of storage for serving blocks anyway.
- [ ] trigger `on_new_epoch` hook on receiving the first block of the next epoch.
- [ ] count sealed blocks on best chain selection.

## spacejam/core

This library contains the core logic of SpaceJam according to the graypaper

| Module        | Description                |
| ------------- | -------------------------- |
| `/`           | Data types                 |
| `/runtime`    | Runtime logic              |
| `/runtime/tx` | State transition functions |

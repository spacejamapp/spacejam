# Block Authoring

This document outlines the implementation plan for block authoring in SpaceJam.

## Overview

Block authoring in SpaceJam consists of several key components:

- [ ] Launch tiny network (6 validators) with docker compose
- [ ] Mock the full network ( assign to Alice, author empty block, empty accumulation, empty transactions, import blocks from network )
- [ ] Validator selection and chain selection
- [ ] Network propagation
- [ ] Block author service
- [ ] Transaction pool management
- [ ] Transaction sender client (testing)
- [x] Block sealing with VRF

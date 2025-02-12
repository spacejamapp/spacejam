# Block Authoring

This document outlines the implementation plan for block authoring in SpaceJam.

## Overview

Block authoring in SpaceJam consists of several key components:

- [x] Launch tiny network (6 validators) with docker compose
- [x] Mock the full network ( assign to Alice, author empty block, empty accumulation, empty transactions, import blocks from network )
- [ ] Transaction pool management
- [x] Validator selection (mb require sort of auto send tx?)
- [x] Block authoring service
- [ ] Network propagation (jam-np)
- [x] Block sealing with VRF

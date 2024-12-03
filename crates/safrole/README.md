1. Entropy Accumulation Calculation (6.22)

- **Chapter**: 6.4 Sealing and Entropy Accumulation
- **Formula**: 6.22

| Symbol | Description               |
| ------ | ------------------------- |
| η      | entropy accumulator (eta) |
| H      | blake2b                   |
| Y      | vrf                       |

```
η'_0 = H(η_0 || Y(H_v))
```

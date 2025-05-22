## TODOs

- [ ] introduce author / importer mode in the spacejam binary

## Spacejam

Library for the SpaceJam node, which mainly connects JAM runtime with p2p network.

## Scripts

connecting with polkajam

```bash
RUST_LOG=jam_node::net=trace polkajam -c=dev run -d 0 --dev-validator 0
```

```bash
RUST_LOG=jam_node::net=trace polkajam -c=dev run -d 1 --dev-validator 1 --port 40001 --rpc-port 19801
```

```bash
RUST_LOG=jam_node::net=trace polkajam -c=dev run -d 2 --dev-validator 2 --port 40002 --rpc-port 19802
```

```bash
RUST_LOG=jam_node::net=trace polkajam -c=dev run -d 3 --dev-validator 3 --port 40003 --rpc-port 19803
```

```bash
RUST_LOG=jam_node::net=trace polkajam -c=dev run -d 4 --dev-validator 4 --port 40004 --rpc-port 19804
```

```bash
RUST_LOG=spacejam,runtime=trace ~/code/spacejam/target/release/spacejam -d 5 run --validator 5 --bootnodes e3r2oc62zwfj3crnuifuvsxvbtlzetk4o5qyhetkhagsc2fgl2oka@127.0.0.1:40000 --bootnodes ecjn4brac2kgu25kiykefww6p6ai7noueo6p5af5tnwjgra4eisya@127.0.0.1:40001 --bootnodes egxdzq3l6mlws7rvweuyeajlg4dszlgyj7hdt3vs2byluzyz3pgaa@127.0.0.1:40002 --bootnodes etfybolcworlsmfsgbkquqwx3yls5vtau263cenebgomamz3xgn2b@127.0.0.1:40003 --bootnodes e427idswxpkliqdinm43ajqbuiwun46c2cuth3w7u7goybzcm277b@127.0.0.1:40004
```

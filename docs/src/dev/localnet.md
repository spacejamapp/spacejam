# Localnet

## Docker

for launching a localnet, we can use the docker compose file provided in the root
directory.

## Binary

- Node-0

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 0 --genesis genesis.json --db spacejam.db/0 --address 0.0.0.0:60000
```

- Node-1

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 1 --genesis genesis.json --bootstrap ehnvcppgow2sc2yvdvdicu3ynonsteflxdxrehjr2ybekdc2z3iuq@127.0.0.1:60000 --db spacejam.db/1 --address 0.0.0.0:60001
```

- Node-2

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 2 --genesis genesis.json --bootstrap ehnvcppgow2sc2yvdvdicu3ynonsteflxdxrehjr2ybekdc2z3iuq@127.0.0.1:60000 --bootstrap erkeohxlubhyzl7ks3mwtzos5olfgocn7dwkbeg7toseadnapn5oa@127.0.0.1:60001 --db spacejam.db/2 --address 0.0.0.0:60002
```

- Node-3

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 3 --genesis genesis.json --bootstrap ehnvcppgow2sc2yvdvdicu3ynonsteflxdxrehjr2ybekdc2z3iuq@127.0.0.1:60000 --bootstrap erkeohxlubhyzl7ks3mwtzos5olfgocn7dwkbeg7toseadnapn5oa@127.0.0.1:60001 --bootstrap  eqe4xodvipulv6vvdkrtmgtd6ztfy3curwtxdpis56yhvxd6jwoka@127.0.0.1:60002 --db spacejam.db/3 --address 0.0.0.0:60003
```

- Node-4

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 4 --genesis genesis.json --bootstrap ehnvcppgow2sc2yvdvdicu3ynonsteflxdxrehjr2ybekdc2z3iuq@127.0.0.1:60000 --bootstrap erkeohxlubhyzl7ks3mwtzos5olfgocn7dwkbeg7toseadnapn5oa@127.0.0.1:60001 --bootstrap eqe4xodvipulv6vvdkrtmgtd6ztfy3curwtxdpis56yhvxd6jwoka@127.0.0.1:60002 --bootstrap e5vesrrri2hbmn2xjam4jawmvmeuvsjz2lrr7snrwyfdbjlehg7iq@127.0.0.1:60003 --db spacejam.db/4 --address 0.0.0.0:60004
```

- Node-5

```bash
RUST_LOG=spacejam ./target/release/spacejam spawn --validator 5 --genesis genesis.json --bootstrap ehnvcppgow2sc2yvdvdicu3ynonsteflxdxrehjr2ybekdc2z3iuq@127.0.0.1:60000 --bootstrap  erkeohxlubhyzl7ks3mwtzos5olfgocn7dwkbeg7toseadnapn5oa@127.0.0.1:60001 --bootstrap eqe4xodvipulv6vvdkrtmgtd6ztfy3curwtxdpis56yhvxd6jwoka@127.0.0.1:60002  --bootstrap e5vesrrri2hbmn2xjam4jawmvmeuvsjz2lrr7snrwyfdbjlehg7iq@127.0.0.1:60003 --bootstrap ezkj2yfyfdbyhdvt3qpd76dx6qeeor3cfgblv25zgq6jthw62xz6a@127.0.0.1:60004 --db spacejam.db/5 --address 0.0.0.0:60005
```

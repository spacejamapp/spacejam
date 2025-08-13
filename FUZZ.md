## Fuzzing reports

### 3. Gurantee signature verification failed

| Report             | Problem                                |
| ------------------ | -------------------------------------- |
| [duna-05][duna-05] | Gurantee signature verification failed |

Got `BadSignature`, could be caused by we are not using the correct validator set.

### 2. Empty Pending reports

| Report             | Problem               |
| ------------------ | --------------------- |
| [duna-06][duna-06] | Empty Pending reports |

Got `CoreNotEngaged` since pending reports is empty.

### 1. Privileges mismatched

| Report                                                   | Problem               |
| -------------------------------------------------------- | --------------------- |
| [zig-10][zig-10], [duna-08][duna-08], [java-11][java-11] | Privileges mismatched |

privileges mismatched, could be problem on our side.

### 0. Incorrect order of next validators

| Report                               | Problem                            |
| ------------------------------------ | ---------------------------------- |
| [zig-09][zig-09], [java-12][java-12] | Incorrect order of next validators |

all state matches except the `next validators`.

```json
{
  "polkajam-ed25519": [
    "0xab0084d01534b31c1dd87c81645fd762482a90027754041ca1b56133d0466c06",
    "0xad93247bd01307550ec7acd757ce6fb805fcf73db364063265b30a949e90d933",
    "0xf30aa5444688b3cab47697b37d5cac5707bb3289e986b19b17db437206931a8d",
    "0x8b8c5d436f92ecf605421e873a99ec528761eb52a88a2f9a057b3b3003e6f32a",
    "0x4418fb8c85bb3985394a8c2756d3643457ce614546202a2f50b093d762499ace",
    "0xcab2b9ff25c2410fbe9b8a717abb298c716a03983c98ceb4def2087500b8e341"
  ],
  "spacejam-ed25519": [
    "0xf30aa5444688b3cab47697b37d5cac5707bb3289e986b19b17db437206931a8d",
    "0xcab2b9ff25c2410fbe9b8a717abb298c716a03983c98ceb4def2087500b8e341",
    "0x4418fb8c85bb3985394a8c2756d3643457ce614546202a2f50b093d762499ace",
    "0xab0084d01534b31c1dd87c81645fd762482a90027754041ca1b56133d0466c06",
    "0xad93247bd01307550ec7acd757ce6fb805fcf73db364063265b30a949e90d933",
    "0x8b8c5d436f92ecf605421e873a99ec528761eb52a88a2f9a057b3b3003e6f32a"
  ]
}
```

`polkajam` seems not have sorting for the next validators, so does `spacejam`, however don't get why
there is a difference in the order of the validators.

[duna-05]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/jamduna/fixed/jam-duna-target-v0.7-0.6.7_gp-0.6.7/1754982087/00000005.json
[duna-06]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/jamduna/jam-duna-target-v0.8-0.6.7_gp-0.6.7/1754982630/00000008.json
[duna-08]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/jamduna/jam-duna-target-v0.8-0.6.7_gp-0.6.7/1754982630/00000008.json
[zig-09]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/jamzig/jamzig-target-0.1.0_gp-0.6.7/1754988078/00000009.json
[zig-10]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/jamzig/jamzig-target-0.1.0_gp-0.6.7/1754988078/00000010.json
[java-11]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754990132/00000011.json
[java-12]: https://github.com/davxy/jam-conformance/blob/main/fuzz-reports/javajam/javajam-0.6.7_gp-0.6.7/1754990132/00000012.json

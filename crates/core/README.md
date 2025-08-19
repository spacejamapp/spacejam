## spacejam/core

This library contains the core logic of SpaceJam according to the graypaper

| Module        | Description                |
| ------------- | -------------------------- |
| `/`           | Data types                 |
| `/runtime`    | Runtime logic              |
| `/runtime/tx` | State transition functions |

────────────
Summary [ 1.154s] 71 tests run: 66 passed, 5 failed, 1434 skipped
FAIL [ 0.141s] spacejam-testing traces::fuzz::test_1754982087_00000005
FAIL [ 0.122s] spacejam-testing traces::fuzz::test_1755248982_00000004
FAIL [ 0.117s] spacejam-testing traces::fuzz::test_1755530300_00000005
FAIL [ 0.107s] spacejam-testing traces::fuzz::test_1755530509_00000004
FAIL [ 0.110s] spacejam-testing traces::fuzz::test_1755531265_00000008

──────────── with out compact encoding
Summary [ 1.186s] 71 tests run: 67 passed, 4 failed, 1434 skipped
FAIL [ 0.130s] spacejam-testing traces::fuzz::test_1755248982_00000004
FAIL [ 0.124s] spacejam-testing traces::fuzz::test_1755530300_00000005
FAIL [ 0.069s] spacejam-testing traces::fuzz::test_1755530509_00000004
FAIL [ 0.096s] spacejam-testing traces::fuzz::test_1755531265_00000008

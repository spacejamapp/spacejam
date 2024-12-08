## Merkle Mountain Range

```text
                      14
                  /        \
               13           6
              /   \       /   \
     17     12     9     5     2
    /  \   /  \   / \   / \   /  \
18 16  15 11  10 8   7 4   3 1    0
```

start -> 2^0 - 1
size -> mmr_size(2^1 - 2)
index: 0, pos: 0, size: 1, peaks: 1

start -> 2^1 - 1
size -> mmr_size(2^2 - 2)
index: 1, pos: 1, size: 3, peaks: 2
index: 2, pos: 3, size: 4, peaks: 2

start -> 2^2 - 1
size -> mmr_size(2^3 - 2)
index: 3, pos: 4, size: 7, peaks: 3
index: 4, pos: 7, size: 8, peaks: 3
index: 5, pos: 8, size: 10, peaks: 3
index: 6, pos: 10, size: 11, peaks: 3

start -> 2^3 - 1
size -> mmr_size(2^4 - 2)
index: 7

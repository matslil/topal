# Owned user-memory model

This independent model exercises `TOPAL-SYSTEMS-USER-EXTENT-001` and the
[design](../../design/owned-user-extents.md). Its immutable inputs describe
already established table, backing and entry observations. Constructing those
Python objects does not create kernel authority. No user buffer is read or
written. Native ownership, all actual syscall callers, source/native/boot
publication and production execution remain separate stages.

From the repository root:

```sh
scripts/run_bounded.py -- python3 -m unittest discover -s linux-kernel/tools/user-memory -v
```

The cases cover complete checked words, no-touch empty ranges, noncanonical and
overflowing candidates, arbitrary mapped virtual placements, a 400-page extent,
all 8192 combinations of direction and four-level P/RW/US grants, holes at every
level and in the middle, page/directory/PDPT/PML4 crossings, independent owner
write rights, foreign/stale/unpinned/aliased owners, unsupported entry/mode
formats, hardware A/D and NX, and immutable snapshots. Read traces prove that a
foreign next-table candidate is not fetched in the model. Broken trusted owner
or entry invariants raise a distinct invariant failure rather than becoming a
normal pointer miss or successful transfer.

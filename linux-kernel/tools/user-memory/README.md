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

## Conditional linked callee inspection

[inspect_callee_graph.py](inspect_callee_graph.py) reads `kernel.elf`,
`process-access-inventory.json` and `callback-bindings.json` from a build folder,
plus complete LLVM/GNU listings. Use LLVM `-d -z --x86-asm-syntax=intel` and GNU
`-d -z -w -Mintel --insn-width=16` output for the same kernel. An optional
`--recovery ADDRESS LLVM_WINDOW GNU_WINDOW` binds a 128-byte targeted window;
the final fifteen bytes remain unapplied to avoid stop-truncated prefix rows.

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/inspect_callee_graph.py BUILD LLVM_LOG GNU_LOG OUTPUT
```

[callee_control.py](callee_control.py) checks complete bytes/control classes,
encoded direct destinations, selected guarded jump-table patterns and bypasses.
Callback targets are checked against the actual linked root call, argument
pushes and registration slices. The reader preserves ambiguous local symbol
identities by address. Inconsistent byte/control witnesses reject; unsupported
starts/calls remain explicit frontiers. Passing means a consistent saved-code report;
`selected_edge_set_closed` remains conditional on its declared entry, callback,
code/table/stack-stability and call-return assumptions. The report retains
`qualification_complete=false`; it grants no memory, entry or cleanup authority.

[test_callee_control.py](test_callee_control.py) supplies sixteen host-only
counterexample tests. The [closed archive](results/stage-3-callee-graph/report.json)
and [review](results/stage-3-callee-graph-review.json) bind 71,457 reached
instructions, 842 function symbol identities, six callback sites and six tables
with 98 entries. Full suites contain 219 user-memory and 82 process-memory
models. Frozen replay reproduces the graph/models and the retained original
recovery-reader failure; it runs no VM, decoder process or compiler build.
The earlier full compiler receipt is identified over unchanged inputs, not rerun.

Offsets 88,215/94,687 remain native coverage gaps. Absence from this graph is not
global unreachability, and presence is not path feasibility. Complete runtime
entries, effects, mutation/lifetime/return/ownership, production and full Linux
qualification remain separate blockers.

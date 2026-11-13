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

## Saved boot and initial interrupt entries

[inspect_boot_entries.py](inspect_boot_entries.py) and
[boot_layout.py](boot_layout.py) bind the private saved `bzImage` to its actual
ELF load segments, zero BSS, protected-to-long transition, GDT, IDT descriptor,
four installed gates and initial paging bytes. They accept only the selected
layout and run no guest. The component image has no production boot-provenance
record; these structural checks do not manufacture one.

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/inspect_boot_entries.py BUILD/kernel.elf BUILD/bzImage OUTPUT
```

Pass `--boot-image BUILD/bzImage` to the callee inspector to rederive the linked
boot handoff and four initial IDT targets as additional roots. They are checked
from actual image/ELF bytes, rather than trusted from a JSON target list. The
[composable evidence archive](results/stage-3-boot-entries/report.json) and
[review](results/stage-3-boot-entries-review.json) retain 206,533 instructions,
1,230,917 bytes, 995 function identities and nineteen declared roots. The same
six guarded tables and 98 entries remain conditional on the earlier assumptions.
The two pinned predecessor archives are explicit replay dependencies.

[run_boot_entry_stage.py](run_boot_entry_stage.py) restores those pinned native
and decoder inputs and runs fresh readback, thirteen offline image corruption
controls, 229 user-memory models and 82 process-memory models. It writes under
the kernel cache and requires the normal bounded cgroup. It invokes no VM,
compiler or disassembler. Ten new structure counterexample tests live in
[test_boot_layout.py](test_boot_layout.py).

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_boot_entry_stage.py linux-kernel/.cache/x86_64/NEW_OUTPUT
```

The saved header advertises `XLF_KERNEL_64`, whose independent entry is at
payload offset `0x200`. That location instead starts 3,584 zero bytes. It is
distinct from the internal transition destination at offset `0x40`. This is a
boot-interface defect to repair; the reader does not qualify that route.
The [Linux boot protocol](https://docs.kernel.org/arch/x86/boot.html)
defines the separate entry; the [Intel SDM, Volume 3A](https://cdrdv2.intel.com/v1/dl/getContent/671190),
section 7.14.1 and Figure 7-8, define the sixteen-byte gates and target fields.

Initial supervisor writable identity mappings do not prove runtime W^X,
ownership or lifetime. The 252 zero IDT slots do not prove those events excluded
or safely handled. Real-mode/loader/CPU prerequisites, complete handler effects,
TSS/stack switching, runtime entry/mutation, AP/DMA and production reconstruction
remain open. Decoder disagreement 45,861,106 is outside this graph and remains
unresolved; native helper gaps 88,215/94,687 and 89/91 coverage are unchanged.

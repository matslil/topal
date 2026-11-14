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

The saved pre-correction header advertises `XLF_KERNEL_64`, whose independent entry is at
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
remain open. Decoder disagreement 45,861,106 is outside this graph. The subsequent
[calendar-data audit](results/stage-3-calendar-data.md) identifies its data/consumer
context while retaining the differing listings and alternate-entry obligations.
Native helper gaps 88,215/94,687 and 89/91 coverage are unchanged.

## Boot-header advertisement correction

Adapter revision **154** clears that unsupported entry flag. Its implemented
setup/protected route still enters the same x86-64 kernel. This follows
`TOPAL-SYSTEMS-ARTIFACT-001`, `TOPAL-COMP-SYSTEMS-ARTIFACT-001` and
`LK-X64-BOOT-001`; it does not implement the separate 64-bit loader route.

The [correction archive](results/stage-3-boot-header-correction/report.json) and
[review](results/stage-3-boot-header-correction-review.json) retain one fresh
source-built process-MM Q35 regression, two targeted compiler tests, 230
user-memory tests, 82 process-memory tests and the workspace/ABI/format/size
checks. The 743 watched inputs stay stable. The kernel remains byte-identical
(`b8212e5e...`); the entire 45,243,376-byte image changes only at file offset
`0x236`, from one to zero. Saved readback and frozen replay check the actual
owner/publication history and reproduce the corrected boot report.

The initial full compiler run was intentionally stopped after review identified
legacy debugger-injected inconsistent kernel-state cases in
[systems_frame_memory_tests.rs](../../../src/topal-compiler/src/systems_frame_memory_tests.rs)
and its fault inspector. Its exact driver, partial results and pre-stop counters
are retained. This was a validation-selection correction, with no AI safeguard
notice; the reach of individual injected cases before stopping is unestablished.
The full suite is **incomplete**, not passed. The selected native regression
uses ordinary source-built syscalls and read-only state observation. Model-level
counterexamples and that regression do not discharge all remaining native fatal,
exception or inconsistent-state obligations. Future qualification must account
for those gaps explicitly.

The native component still has no production boot-provenance record. Its
nonfatal GDB `.debug_names` warning is retained; this stage grants no new debug
format qualification. Separate loader support, complete entries/effects/lifetime,
production reconstruction and full Linux acceptance remain open.

## Saved gateway admission model

[gateway_contract.py](gateway_contract.py) defines an independent predicate for
the selected `brk`, exec-reset and process-completion pre-invocation checks.
[gateway_machine.py](gateway_machine.py) interprets a strict subset of the
actual saved gateway bytes in Python. Unsupported instructions, inaccessible
model words and unexpected control flow raise a coverage gap. It models only
CF/ZF and IF/DF observations; the temporary flags word is a projection, not an
exact native stack observation. No machine code or guest runs.

[inspect_gateway_admission.py](inspect_gateway_admission.py) binds the gateway,
allocated data symbols, actual relative call/return sites, refusal marker and
pre-save boundary to the ELF before comparing those independent descriptions.
The report always retains `qualification_complete=false`. Source dictionaries
and numeric matches create no owner, caller or entry authority.

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/inspect_gateway_admission.py KERNEL_ELF OUTPUT
```

The [composable archive](results/stage-3-gateway-admission/report.json) and
[review](results/stage-3-gateway-admission-review.json) retain **5,126 cases per
kernel**, across the current process-MM kernel and three retained predecessors:
**20,504 interpretations**, with 1,173 reaching the modeled admission boundary
and 3,953 the refusal marker in each kernel. Thirty-six Python-only weakened-
guard controls are detected. Eighteen new counterexample tests bring the full
suites to **248 user-memory** and **82 process-memory** tests. Frozen replay
reproduces the four reports, controls, retained inventory and both suites.
The pinned boot-header-correction archive is an explicit source dependency.

Read-only inspection also establishes that the interrupted compiler run had
completed all twelve diagnostic modes on each of three predecessor kernels:
27 pre-entry refusals, six post-write fatal cases and three ordinary exhaustion
cases. The three corresponding individual Rust tests have `... ok` lines in
the retained partial log. Earlier reporting left their reach unestablished;
the newly inventoried files resolve that uncertainty. These are original
producer assertions. Complete CPU inputs and every before/after byte were not
retained, so this is not independent replay of all native semantics or fresh
execution on the current kernel. The full compiler command remains interrupted.

No legacy test, kernel check or production gate is removed. Actual ownership,
fresh nonwrapping entry generation, caller origin, hardware/mapping/lifetime,
asynchronous behavior, other gateway operation classes and post-admission fatal
effects remain separate obligations. Interpreter arithmetic/conditions follow
the [Intel SDM](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html),
Volume 2A (CMP/Jcc and encoding) and Volume 2B (MOV/RDMSR/PUSHFQ). This model is
an x86 adapter observation, not a portable language rule or activation authority.

## Separated calendar data

[inspect_calendar_placement.py](inspect_calendar_placement.py) checks independent
Gregorian object/linked bytes, read-only ELF sections and loads, exact process
layout transformation and all external bindings. Its four byte-model tests
reject unplanned changes without executing altered bytes.

The [placement report](results/stage-3-calendar-placement.md) binds provider
revision 235 to fresh child exec and the complete 51-workload pinned corpus,
341 models, selected compiler checks and exact frozen source/data replay.
It also preserves fresh decoder agreement, current-corpus 66/91 helper-entry
coverage and actual writable supervisor mapping samples with unresolved final
root coverage. It grants no hardware-protection or production qualification.

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_calendar_placement.py linux-kernel/.cache/x86_64/NEW_PLACEMENT_OUTPUT
```

This native driver currently uses the preceding local
`process-mm-applications-corpus/run.json` cache to locate the pinned image.
The report's retained frozen replay works from committed archives without that
original cache. This driver is not a cold production-reconstruction path.

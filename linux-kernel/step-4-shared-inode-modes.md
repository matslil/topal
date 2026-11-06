# Step 4: shared inode mode actions

`TK-DEC-280`; follows the [systems design](design/shared-inode-mode-actions.md)
and `TOPAL-SYSTEMS-INODE-MODE-{STATE,ABI,ACTION}-001`.

## Stage 1: definition and shared-state model

The [portable metadata representation](../src/topal-semantics/src/systems_inode_metadata.rs)
owns the real four-word byte block used by model observations, rather than a
candidate buffer. Existing regular objects, the mutable directory and pipes now
use it for creation, metadata, transfers, truncation, unlink and termination.
Description access rights and positions remain separate from inode permissions.
Pipe creation time remains separate from mutable ctime. Existing model public
observations retain their meanings; pipe observations additionally expose mode
and ctime, and directory metadata has an owned snapshot with no mutation grant.

The admitted prepared-root operation directly updates this shared model state.
It preserves file type and atime/mtime while replacing low 07777 permissions
and recording the supplied mutation-clock observation as ctime. This is a model
of the reference law, not opaque source execution or complete native admission.
The later source action still needs actual-owner loan/exclusion, Boolean
completion and complete reclamation qualification. There is no compiled chmod
handler or new VM claim in this stage.

All **199 semantics tests pass**, including nine new tests:

- All 4096 permission patterns on each of regular/directory/FIFO types preserve
  the exact other bytes, with high native argument bits and full-u64 clock data.
- Repeated equal mode updates ctime, preserves other times and supports complete
  logical metadata erasure.
- Read-only independent and inherited descriptions see the shared mode without
  changing granted rights, contents, allocation, aliases or positions.
- Unlinked old inodes remain distinct from recreated names; closed linked objects
  remain mutable by name, stale descriptions cannot affect reused slots.
- Directory chmod leaves entries/mtime unchanged; later namespace mutations
  preserve the mode and update parent times; termination retires the metadata.
- Wrong owner, same-label foreign pool, invalid name, stale reference and
  poisoned existing controller admissions preserve complete filesystem state.
- Both pipe endpoints and inherited aliases share mode/ctime while preserving
  the complete FIFO, creation time, readiness, counters and endpoint ownership.
- Chmod does not wake a blocked reader; peer close still does. Reused pipe slots
  receive fresh metadata and stale endpoint generations cannot mutate them.
- Equal pipe labels, generations and tickets cannot cross actual pool identity.

The [primary research record](knowledge/x86_64/inode-mode-research.json) binds
seven complete Linux 7.2.9 source files. The actual source archive SHA-256 matches
the pinned manifest, and every researched cached file matches its archive
member byte for byte. The research records syscall numbers, flags, native mode
adaptation, path acquisition and attribute selection. Source research is not
execution against the Linux reference VM; adapter/error-order comparisons remain.

After the user freed memory, the unchanged bounded wrapper admitted the initial
workspace retry with **3072 MiB**, zero swap and **1569 MiB host reserve**; it
passed. The changed-model locked all-target workspace check also passes with
3039 MiB, and the final 199 model tests passed with 3034 MiB. All four
specification/requirement coverage checks pass with 305 unique systems rules.
The first coverage attempt ran before the new traceability row was written and
reported those three missing requirement links; the completed row and full
rerun pass. Formatting, the existing 2000-line Rust ceiling, 478 local links in
changed documents and raw trailing/end/space-before-tab checks pass.
No wrapper limit, host reserve, quota or security policy was relaxed.

Risk is high because shared lifetime, authority, mode bits and native error order
interact. The current review compares all affected model consumers with the
approved design and existing laws, then uses exhaustive independent byte and
complete-state oracles. Pinned Linux execution, independent source/native/GDB
and physical observations are required in subsequent stages. The previous
[TK279 checkpoint](step-4-unsigned-region-fields.md) retains its distinct 2280
Rust tests and 158 VM observations; those counts do not qualify this new action.

## Stage 2: ordinary source and actual-owner components

The [entry](kernel/policy/inode-mode-action.t) imports the ordinary
[metadata module](kernel/support/inode-metadata.t). It checks the complete
32-byte extent, permission/time ranges and existing type before any store.
Ordinary quotient/multiplication retains the existing type prefix; checked
eight-byte little-endian stores update mode and ctime. The action constructs
no pointer or grant, changes no core language meaning and uses no native
mutation substitute. Read-only or malformed data returns false without stores.
An independently changed false-after-mode body preserves that earlier store,
without pretending to roll it back.

The exact instance is `inode-metadata`, with two Nat inputs and Boolean result,
unsigned source schema `/4` and the unchanged frame ABI/family. Full source,
module snapshot, runtime, code/read-only data, hooks and bounds are reproduced
and verified. Its complete temporary certificate is **12208 bytes**:
12064 source evaluation, 112 input/view bridge and 32 header. The reviewed
ordinary numeric operation accounting includes division's full bounded
allocation paths, rather than assuming a machine-word quotient. The maximum
operand bound is 65 bits, one view is issued and all 47 retained native
functions have stack evidence; complete source stack is **1296 bytes**.
The existing 65536/4096 limits are unchanged. Observed peaks of 7328 in the
full data corpus and 936 in debugger samples do not replace the certificate.
Owning syscall/controller stack paths remain a later qualification obligation.

The model additionally lends the same live inode block used by regular,
directory and pipe observations. Actual byte operations occur on that block,
not a copied candidate. Independent filesystem and pipe domains validate
real references and mode encoding before publishing a fresh invocation;
permission facts outside 07777 and exhausted generations preserve state.
Nested helper views retain owner-relative effects and current-frame exclusion.
History survives intermediate object destruction and slot reuse. Same-label/
generation inode, position and page views remain distinct while filesystem
effects conservatively conflict in their common controller domain.

All **208 semantics tests pass**, with nine further loan tests. True and exact
mode/ctime plus unchanged atime/mtime are required after all model reclamation
obligations. Each missing cleanup obligation, false after stores, wrong retained
field, partial update and abandonment poisons the owner without undoing successful
stores. Normal access and new loans then reject; terminal reclamation can retire
the poisoned owner. Direct model operations, model proof flags and terminal
retirement do not prove source execution or physical erasure.

Four independent compiler component tests cover:

- The [C caller](labs/qemu/x86_64/inode_mode_client.c) executes **36906 calls**:
  all 4096 permission patterns on regular/directory/FIFO types at each of three
  placements, including an unaligned field and both guarded-page boundaries,
  then 42 data refusals. Its separate division-based byte oracle checks the
  entire payload page, preserved type/times and full unsigned clock values.
- Thirty-two narrow extents and invalid permission facts point at inaccessible
  payload pages. Invalid types use physically read-only pages; read-only views
  preserve valid metadata. Four additional compiled probes construct actual
  65/256-bit permission/time facts and refuse before inaccessible payload access.
- Sixteen malformed native entry/control/frame cases reach the abort hook before
  changing the complete control, arena or payload. Fourteen changed artifact/
  caller/runtime/cost/effect cases reject reproduction, including coordinated
  schema downgrade plus runtime-digest removal. Changed source encoding fails
  the independent byte oracle; a false-after-first-store variant proves no
  rollback independently of the model.
- The [debugger oracle](labs/qemu/x86_64/inode_mode_debug.py) observes **nine calls
  and nine arena/frame erasures**, all actual entry/imported-helper facts,
  computed type prefixes, full source mode/time values, internal Boolean low-bit
  returns and exact public Boolean results. It observes 48 owner-payload byte
  reads and 64 stores, no access to atime/mtime, unchanged control generation
  laws, all six callee GPRs, all sixteen XMM registers, restored RSP and contained
  source-stack use. Even the equal-mode/equal-clock case performs its stores.

The C component owns and physically erases its full arena and immutable input
staging, with independent debugger readback. It is not the kernel controller:
kernel retired-stack erasure, actual subsystem admission and cleanup still need
fresh machine observations. The selected kernel and its 61-operation dispatch
remain unchanged in this component stage.

Early source attempts used an unsupported nested branch spelling, unqualified
module reference and an ambiguous unbound application/arithmetic expression.
Using existing helper functions, fully qualified module calls and explicit
ordinary value bindings fixes them without a language change. The first GDB
attempt encountered an optimized-out C-only observation variable; making that
observation volatile preserves it, and the complete debugger rerun passes.

Final component-stage validation passes all **263 compiler unit tests** and all
**208 semantics tests**, with no failed or ignored tests. Locked workspace
all-target checking, all four specification/requirement coverage checks,
bounded formatting and the unchanged 2000-line Rust ceiling pass. All 305
systems norms remain unique, all 406 local links in this stage's changed
documents resolve, and raw whitespace/debugger-syntax checks pass. The normal
bounded wrapper retains its host reserve and zero swap throughout. Complete
workspace test/doc and changed-kernel machine qualification remain Stage 4 work;
this stage does not reuse the preceding kernel's VM results as new evidence.

## Remaining staged work

Stage 3 must integrate all four chmod-family adapters and the real persistent
metadata owners, new exact layout versions, Linux reference controls, physical
cleanup and unmodified BusyBox workloads. Stage 4 must complete the affected
Rust/lab/native/VM cohort and archive exact evidence.

No core language change is needed for this design. General credentials,
filesystem/device services, dynamic programs, distribution replacement,
containers and VM hosting remain part of the original unfinished acceptance.

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

## Stage 3: native adapters and actual inode execution

The selected root now declares `inode-mode-library` and the exact
`static-linux-dispatch-v5` vocabulary: 65 operations plus Unsupported. The four
new operations route chmod, fchmod, fchmodat and fchmodat2 through Linux argument
adaptation, copied-name/live-description lookup and admitted clock acquisition
before the source receives authority. Earlier four source vocabularies and named
adapters remain frozen; the native selector oracle checks all five, including
holes and wide inputs, in original and reversed Enum orders.

The [native controller](../src/topal-compiler/src/systems_inode_region.rs) lends
the actual regular-object, directory or pipe metadata block. It checks the live
controller, exact record geometry, object/generation/endpoint ownership, canonical
type, permissions, masked execution context and clean invocation state. The
ordinary source alone performs the metadata stores. Complete post-state checking
requires its true result, preserved type/atime/mtime and exact permissions/ctime.
The 1296-byte source stack plus 184-byte live controller frame and 64-byte
alignment allowance yields a **1544-byte combined bound**. No resource limit grew.

Regular metadata now owns the complete type/permission word at object offset 24;
stat publishes that word directly. Directory metadata occupies header bytes
32–63, with the old timestamp words reserved zero. Pipe mode/times occupy bytes
64–95, with separate creation time at 96. Creation, stat, namespace mutation,
transfer and terminal consumers use these placements. The new filesystem
evidence schemas are `/4` with the source page pool and `/5` with native page
allocation. Owner extent and child snapshot boundaries remain unchanged.
The [boot binding tests](../src/topal-compiler/src/systems_inode_binding_tests.rs)
reject changed layout, source/runtime/cost facts, missing or cross-instance
bindings, explicit null fields and modified controller/source instructions.

The [pinned Linux control](labs/qemu/x86_64/results/topal-inode-mode-linux-control.json)
passes **33 independently issued cases** on Linux 7.2.9. It checks mode/fd/flag
adaptation, ignored fchmodat argument four, empty-name regular/directory/pipe
selection, read-only description rights, alias/unlink behavior and exact errors.
A disposable guest tmpfs remount checks lookup-before-EROFS; the host is unchanged.

The [fresh physical suite](labs/qemu/x86_64/results/tk280/stage-3/inode-qualification.json)
passes **12 VM dispositions**:

- Primary: 82 caller returns and 14 actual source actions across all four calls,
  read-only/reopened/duplicated descriptions, closed linked names, unlink/recreate,
  object reuse, mutable directory metadata, both pipe endpoints and ABI errors.
- Lifetime: 131 caller returns and 20 actions, including failed/successful exec,
  close-on-exec filtering, inherited independent/aliased descriptions, child
  file/pipe mode changes, shared unlink/recreate and equal-mode stores.
- Controlled inactive-child termination: 106 observed returns and 16 actions,
  with complete live/inactive process backing and invocation history erased.
- Seven pre-entry controls reject a foreign owner, record misalignment, enabled
  interrupts, excessive adapted permissions, dirty arena, exhausted generation
  and malformed inode type. Complete actual owner/runtime state remains unchanged.
- Independently compiled wrong-mode and false-after-first-store sources stop
  after complete cleanup, preserve their earlier effects and never resume the
  owner. Their native root/provider objects are unchanged. QMP independently
  confirms all controlled targets remain stopped after debugger disconnect.

For every successful action, the debugger reads actual ordinary Nat facts and
view identity, observes eight byte loads and sixteen byte stores, observes no
atime/mtime access, and compares the complete process owner against the exact
two-field delta. It verifies caller GPRs, all sixteen XMM registers, RSP, complete
65536-byte arena/control reclamation, the 4096-byte retired stack and 64-byte
input frame before owner return. Generations remain consecutive across object
reuse, exec and child restoration. Terminal cleanup erases history and all
311296 owner bytes. These are physical observations, separate from model proofs.

Five [unchanged BusyBox workloads](labs/qemu/x86_64/results/tk280/stage-3/inode-busybox.json)
pass: numeric and symbolic chmod followed by stat, directory chmod/stat, an
already-open reader after mode zero, and chmod independent of creation umask.
All targeted records bind kernel SHA-256
`d22cb2d7500e7447a780856837d5872c072f20728755380caedb16e78bdae3b1`;
the two deliberately changed source kernels have distinct recorded hashes.
The artifact/root/provider/object/boot/gate epochs are **253/239/240/232/153/253**.
The [archive index](labs/qemu/x86_64/results/tk280/stage-3/archive.json) binds
49 complete, losslessly compressed logs, including the initial failed inactive
host observation. That attempt raced normal serial completion against debugger
disconnect; requiring explicit QMP confirmation fixes the harness, and all
twelve final dispositions pass. No kernel check was weakened.

Current validation passes all **264 compiler unit tests**, **628 language tests**,
**209 semantics tests**, locked workspace all-target checking, four traceability
coverage tests, formatting and the unchanged 2000-line Rust ceiling. The first
compiler run found an old relocation count; the revised test checks all four
page/inode source-presence combinations and their exact closed imports. The
public page-owner test also needed the new native-page evidence version `/5`.
An initial utility launch assumed `/usr/bin/busybox`; supplying the already
pinned binary fixes host input discovery, and all five actual workloads pass.
Normal bounded execution retains the host reserve and zero cgroup swap.

## Stage 4: complete affected qualification

Stage 4 covers the full affected committed Rust/lab/native/VM cohort and its
versioned publication contracts. The 17 targeted Topal VM runs above do not
replace that obligation. TK279's 158 VM observations and complete workspace
results remain historical evidence for its preceding kernel, not this kernel.

The [first full committed Rust run](tools/rust-qualification/results/tk280/before-legacy-position-fixture-fix/qualification.json)
passes all-target checking and the separate doc-test command, but records 2304
passed tests and one failed test across 41 targets. The legacy file-position
snapshot refusal fixture inherited V5's mandatory inode declaration, so its
default empty snapshot bundle reached the earlier inode guard. Explicitly
selecting frozen V4 and omitting the unrelated page/inode slots restores the
fixture's intended position-guard check; the focused test passes. The same test
also verifies that neither unrelated native controller is imported. Production
admission and the selected kernel remain unchanged. Complete logs are preserved;
the [fresh qualification on committed corrected sources](tools/rust-qualification/results/tk280/final/qualification.json)
passes locked workspace all-target checking, **all 2305 tests across 41 targets**
with no failed or ignored tests, and the separate doc-test command. The latter
discovers no executable doc tests. The record binds revision
`86c72c443be8bd993b06c71de44504071ce01a6b`, the exact tree, tool versions,
unchanged bounded wrapper and all three complete, losslessly compressed logs.

The complete affected machine cohort now passes **176 Topal VM observations**.
The completed runner's logs independently count 164 QEMU starts. The twelve
inode dispositions from Stage 3 retain their exact current kernel, source,
caller and observer bindings, independently checked before inclusion; the
targeted five BusyBox cases are rerun within the fresh **45-workload corpus**
and are not counted twice. The 33 Linux reference cases remain a separate
comparison corpus, not Topal VM starts.

| Observation group | Count |
| --- | --- |
| Root, file-position and RAM-page source owners | 43 |
| Shared inode admission, lifetime and refusal controls | 12 |
| Positional, named, file-open and volatile-file behavior | 17 |
| Header, dispatch, identity, retained/structured state, numeric and finite policies | 35 |
| Clean/empty/null execution and 45 external BusyBox workloads | 48 |
| Real/monotonic clocks, child/pair/pipe lifecycle and deadlines | 21 |
| Total | 176 |

The [final manifest and complete logs](labs/qemu/x86_64/results/tk280/final/qualification.json)
bind the primary kernel, exact result hashes, per-command VM counts, runner
source snapshots and corrected observer files. All **43 lab publication
contracts** pass, including the new inode contract, current canonical geometry,
the new dispatch with four frozen compatibility controls, actual ownership,
cleanup, source values and existing Linux-reference manifest contracts. The
complete unittest log is also retained. Observer/fixture and publication changes
are recorded separately from the committed compiler/kernel/source subject;
none of those production inputs changed after the complete Rust qualification.

Three earlier cohort attempts remain archived and excluded from passing counts.
The first could not acquire the normal validation lock inside the sandbox;
normal authorized execution retained the same wrapper and safeguards. The
file-open observer expected a zero directory header, but the canonical header
now contains `/tmp`'s initial mode. Its corrected independent byte oracle checks
the entire initial filesystem and exact no-mutation refusal, including that
mode. The numeric counterexample fixture omitted the mandatory inode source
module. Numeric and finite-policy variants now retain the complete primary
policy tree before changing their intended module; unchanged native-root/
provider and independent rejection checks still pass. No production check was
weakened, and no partial failed attempt is counted as successful.

The systems ledger has **306 unique rules**. Formatting, the unchanged 2000-line
Rust ceiling, local Markdown targets, raw whitespace and lossless log/hash
verification pass. The bounded wrapper continues to adapt to available host
RAM while retaining the normal host reserve and zero swap. No host security,
memory limit, reserve, quota or swap policy was relaxed.

No core language change is needed for this design. General credentials,
filesystem/device services, dynamic programs, distribution replacement,
containers and VM hosting remain part of the original unfinished acceptance.

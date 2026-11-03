# Step 4: independent ordinary region instances

`TK-DEC-276`; implements the [instance design](design/instance-scoped-region-actions.md)
and `TOPAL-SYSTEMS-REGION-INSTANCE-SOURCE-001`,
`TOPAL-SYSTEMS-REGION-INSTANCE-BINDING-001` and
`TOPAL-SYSTEMS-REGION-INSTANCE-ROOT-001`. Step 4 remains in progress; the
[status ledger](step-4-status.md) retains distribution, container and VM-host work.

## Behavior and ownership

Independent source objects can select the same ordinary signature and export
without sharing private names, hooks or caller storage. The new systems context
`borrowed-byte-region-instance-action-v1` retains the existing one-region,
zero-to-sixteen-Nat, Boolean selection and five-argument native frame ABI.
Core syntax, value semantics, arithmetic, effects and nonescape laws are unchanged.

A canonical label determines link ownership: 1–48 ASCII characters, initially
a lowercase letter, then lowercase letters/digits and single internal hyphens.
The source namespace is `topal.region-instance.<label>.`; native entry/hooks
replace hyphens by underscores. Labels create no live memory authority. The
kernel binds the exact label derived from its qualified caller slot, actual
source code/data/hooks and independent owner/control/arena/history evidence.
An arbitrary valid offline object does not acquire a kernel caller grant.

The primary kernel executes both `root-region` with zero input facts and
`file-position` with two. Their private code, literals, hooks and temporary
storage are separate. Calls remain synchronous and nonnested under the existing
controller exclusion; there is no new concurrency or SMP claim.

The [root wrapper](kernel/policy/bootstrap-byte-copy-frame.t) calls the existing
ordinary Unit byte-copy implementation and returns true. Its native caller
passes a null/zero frame, retains the actual Boolean result, revokes views and
erases/readbacks the complete arena/control/retired source stack before requiring
true and resuming the root owner. False preserves successful stores and stops
without owner resumption or rollback. The model records that runtime acceptance
obligation and leaves the source result and owner bytes opaque. The independent
application byte guard remains.

The [file-position algorithm](kernel/policy/file-position.t), two actual facts,
32-byte owned staging, controller ownership, repeated generations and Linux
behavior retain their preceding contracts. Its instance identifies that same
qualified caller domain rather than granting a new path to shared storage.

## Representation and bounds

Instance source objects use schema 2; the private frame ABI and reviewed
adapter family remain version 1. New root and file-position bindings use schema
3, with exact instance/count fields. The root additionally binds 56 live caller
bytes. Explicit nulls cannot erase present context fields; missing/mismatched
fields reject when the selected context requires them. Boot packaging checks
both owning callers before admitting the complete registered instance namespaces.

Legacy Unit schema 3, action schema 1 and unscoped frame schema 1 remain separately
selectable with their old symbols, entry layouts and canonical omitted instance
fields. The Unit root binding remains version 2; action and unscoped-frame
file-position bindings remain versions 1 and 2. New fields cannot silently
reinterpret an old context.

The family digest remains
`c33b5beb0f1869341bf24493e867f5d3101a578a12962b525c2613c1e09e3bbe`.
The root proves 544 source-stack bytes plus 56 live caller and 32 alignment bytes,
632 combined, with a 136-byte complete allocation bound. File-position retains
1856 source-stack bytes, 1976 combined and a 60152-byte allocation bound.
The arena/reserve/function/view/bit limits remain unchanged. Physical depths
are observations within those complete static bounds, rather than substitutes
for them.

## Validation

The complete bounded Rust workspace passes, including 621 language, 254 compiler
unit, 261 compiler CLI and 173 semantics tests, eight action integrations, two
Unit-data integrations and both frame/instance native-GDB integrations. All
workspace targets, workspace formatting and explicit included-source formatting
pass. Artifact/root/provider/object/boot/gate revisions are
250/236/237/229/150/250. All 137 QEMU observations pass, including forty
unmodified BusyBox workloads, nine root and thirteen field cases. The general
positive cohort retains seventy observations sharing the primary kernel and
twenty-one identified variants. All 41 Python lab contracts pass, together with
297 unique normative rules, relative links, exact code/data/manifest
correspondence, instance/caller/resource checks and whitespace. Primary kernel
SHA-256 is
`397e07bfa0c0528e73389ea781e112923b2637a5aa1ee587e4a795cc2cbc7bb0`.

The numeric counterexample initially failed before boot because its minimal
copied library tree omitted the new root wrapper. The shared copy helper now
retains both wrapper and Unit implementation. Numeric and finite-policy
qualification pass after that fixture repair; completed earlier kernel evidence
retains its exact artifacts. This changes the harness's input preparation,
without changing kernel instructions, metadata schemas or resource contracts.

A separate file-size audit fails on six pre-existing oversized Topal files and
ignored cached Linux dependencies. Comparison with the preceding qualified
commit confirms that all six Topal files already exceeded the 2000-line ceiling;
no new instance source file exceeds it. This audit is not a passing validation.

The [native instance integration](tools/toolchain-gate-builder/tests/region_instances.rs)
links two real three-fact source objects with the same public `apply` export.
Each executes 48 full-width/order-sensitive vectors and sixteen refusal cases
using the existing guarded read-only input-frame oracle. GDB observes actual
ordinary parameters, instance-local arenas/data, views, generation, native
Boolean, GPR/SIMD preservation and stack/return. Both objects execute in one
process. Two cross-instance code/control calls must fail before source preparation,
allocation or owner writes; shared child/parent memory checks and both abort
hooks independently check that refusal boundary.

Compiler tests cover canonical labels, distinct complete names, LLVM coexistence,
wrong-context replay, strict serialization, unselected namespace/hooks, caller
slot/count/instance mismatches, actual code/data changes and complete bounds.
Linked storage inspection rejects writable/TLS code, invalid BSS extents and
aliases with other named allocations. Legacy Unit/action/frame tests remain.

The [root VM harness](labs/qemu/x86_64/region_gate.py) observes null/zero transport,
the actual six-frame helper chain, Boolean result, owner bytes, preservation,
cleanup and finalization. Its false-after-store control proves successful stores
survive complete temporary cleanup while owner resumption is refused. Pre-entry,
wrong-byte, absence and explicit legacy Unit controls remain. Controlled cases
wait through successful debugger disconnect and a deliberate 0.2-second delay,
then require QMP's actual stopped/debug state before teardown.

The [field VM harness](labs/qemu/x86_64/file_region_gate.py) preserves the complete
position/alias/append/positional/cleanup oracles, binds the new field instance,
and separately rebuilds and executes both legacy action and unscoped-frame
contexts. Their source ABI, schema, optional-field omission and native execution
are explicit compatibility controls.

Risk is high at caller/code identity, namespace closure, independent storage,
Boolean failure sequencing, legacy representation and complete resource bounds.
Self-review checks the authoritative ordinary and systems intent, exact generated
objects and callers, independent C/GDB/VM observations and negative packaging
controls. Human PR review remains required; no independent reviewer is claimed.
All execution uses isolated QEMU TCG guests and the existing bounded host checks.

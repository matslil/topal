# Linux-kernel project traceability

This file connects the approved project direction to its first artifacts and
future evidence. It does not duplicate the repository-wide language
traceability ledger.

| Project intent | Research/design identity | Current artifact | Required downstream evidence |
| --- | --- | --- | --- |
| implement only x86-64 initially | `LK-SCOPE-001` | [baseline](baseline.md), [approved boot-adapter contract](design/x86-boot-adapter.md), [generated adapter](step-4-x86-boot-adapter.md), [pinned-QEMU gate](step-4-toolchain-gate.md) | initial entry, exception, and kernel-owned-memory execution pass; native UAPI implementation remains |
| support the complete latest interface | `LK-SCOPE-002`, `LK-SCOPE-003` | [baseline](baseline.md), [UAPI map](knowledge/common/linux-uapi.md), [source inventory](inventory/coverage.md) | semantic dispositions and differential tests |
| keep kernel implementation and imports license-clean | `LK-SCOPE-006`, `TK-DEC-011`, `TK-LICENSE-001`, `TK-PROVENANCE-001`, `TK-UAPI-LICENSE-001`, `TK-BINARY-001` | [approved licensing and provenance policy](design/license-and-provenance.md), root `LICENSE`, workspace package metadata | first-party work is Unlicense and clean-room; every future import remains fail-closed pending a digest-bound license review |
| use Topal elements instead of source assembly | `TK-ELEMENT-001`, `TK-ELEMENT-ENTRY-001`, `TK-ELEMENT-MACHINE-001` | [approved systems profile](../docs/systems-profile.md), [formal rules](../spec/systems-profile.md), [checked kernel root](kernel/arch/x86_64/toolchain-gate.t), [generated provider object](step-4-x86-provider-object.md), [linked artifact](step-4-linked-artifact.md), [generated boot adapter](step-4-x86-boot-adapter.md), [ordinary compiler publication](step-4-compiler-publication.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | initial entry roots, machine primitives, adapter transitions, and ordinary compiler publication are typed, inspected, and executable-qualified |
| model hardware/concurrent observations | `TK-ELEMENT-OBSERVATION-001` | `TOPAL-SYSTEMS-OBSERVATION-001`, [Step 4 semantic report](step-4-semantic-foundation.md), [linked artifact](step-4-linked-artifact.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | debug-break transition and context-preserving resume pass under pinned QEMU; other observations remain |
| use bounded kernel-owned bootstrap memory | `TK-ELEMENT-STORAGE-001`, `TK-DEC-008`, `TK-DEC-016` | `TOPAL-SYSTEMS-STORAGE-001`, [bounded-storage report](step-4-bootstrap-storage.md), [bootstrap-region contract](design/bootstrap-region.md), [checked kernel root](kernel/arch/x86_64/toolchain-gate.t), [linked artifact](step-4-linked-artifact.md), [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | allocation/access checking, byte-content model, x86 lowering, structural load/store inspection, and QEMU memory evidence pass |
| refine boot memory without exposing firmware layouts | `TK-KERNEL-BOOT-001`, `TK-KERNEL-MEMORY-001`, `TK-MAP-PHYS-001`, `TK-DEC-017` | `TOPAL-SYSTEMS-BOOT-MEMORY-001`, [boot-memory contract](design/boot-memory-description.md), [implementation report](step-4-boot-memory.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | semantic model, affine checker, bounded BIOS E820 collection, x86 validation witness, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| allocate physical frames without exposing addresses | `TK-KERNEL-MEMORY-001`, `TK-MAP-PHYS-001`, `TK-DEC-018`, `TK-DEC-035`, `TK-DEC-038` | `TOPAL-SYSTEMS-FRAMES-001`, [frame-allocation contract](design/physical-frame-allocation.md), [bounded multi-frame refinement](design/bounded-multi-frame-extent.md), [disjoint live-extent refinement](design/disjoint-live-frame-extent.md), [initial report](step-4-frame-allocation.md), [multi-frame report](step-4-bounded-multi-frame-extent.md), [disjoint-extent report](step-4-disjoint-live-frame-extent.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | architecture-neutral affine model, exact request-shape tracking, sealed one- and three-frame E820 extent selection, one prepared-space-owned extent plus one disjoint source-owned extent, whole-extent release, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass; splitting, dynamic requests, and arbitrary live-extent sets remain |
| access physical frames through opaque kernel mappings | `TK-KERNEL-MEMORY-001`, `TK-MAP-PHYS-001`, `TK-DEC-019` | `TOPAL-SYSTEMS-MAPPING-001`, [mapping contract](design/kernel-mapping.md), [implementation report](step-4-kernel-mapping.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine map/access/unmap model, sealed policy checker, x86 adopted identity mapping, indirect byte access inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| replace bootstrap translations without exposing page-table formats | `TK-KERNEL-MEMORY-001`, `TK-MAP-PHYS-001`, `TK-DEC-020` | `TOPAL-SYSTEMS-MAPPING-001`, [translation-space contract](design/translation-space.md), [implementation report](step-4-translation-space.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine update/commit/activation model, sealed source checker, three generated x86 provider operations, structural page-table/CR3 inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| edit active translations without exposing virtual addresses or invalidation instructions | `TK-KERNEL-MEMORY-001`, `TK-MAP-PHYS-001`, `TK-DEC-021` | `TOPAL-SYSTEMS-MAPPING-001`, [active translation-edit contract](design/active-translation-edit.md), [implementation report](step-4-active-translation-edit.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | exclusive affine map/unmap model, sealed source checker, four generated provider operations, structural NX/publication/invalidation inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| mask local interrupts without exposing target state | `TK-ELEMENT-CRITICAL-001`, `TK-DEC-006`, `TK-DEC-007`, `TK-DEC-022` | `TOPAL-SYSTEMS-CRITICAL-001`, [local-interrupt critical-scope contract](design/local-interrupt-critical-scope.md), [implementation report](step-4-local-interrupt-critical-scope.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine enter/restore model, exact prior-state restoration, sealed source checker, private x86 flags lowering, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| modify CPU-shared words atomically without exposing addresses or instructions | `TK-ELEMENT-ATOMIC-001`, `TK-DEC-007`, `TK-DEC-023` | `TOPAL-SYSTEMS-ATOMIC-001`, `TOPAL-SYSTEMS-ORDER-001`, [atomic-word location contract](design/atomic-word-location.md), [implementation report](step-4-atomic-word-location.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | architecture-neutral location/modification-order model, affine source lifecycle, private x86 locked compare/exchange, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| deliver a local external interrupt without exposing vectors, controllers, or entry frames | `TK-ELEMENT-ENTRY-001`, `TK-ELEMENT-OBSERVATION-001`, `TK-DEC-006`, `TK-DEC-024` | `TOPAL-SYSTEMS-LOCAL-INTERRUPT-001`, [local-notification contract](design/local-notification-interrupt.md), [implementation report](step-4-local-notification-interrupt.md), checked root, x86 provider and boot adapter, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | monotonic affine event model, mandatory completion before resume, private local-APIC self-notification/EOI, generated interrupt gate, exact prior-mask restoration, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| observe monotonic time without exposing counters, registers, or addresses | `TK-ELEMENT-OBSERVATION-001`, `TK-DEC-025` | `TOPAL-SYSTEMS-MONOTONIC-CLOCK-001`, [monotonic-clock contract](design/monotonic-clock-observation.md), [implementation report](step-4-monotonic-clock-observation.md), checked root, x86 provider and boot adapter, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | same-clock nondecreasing model, sealed source sequence, private HPET validation/state, initial and replacement mappings, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| deliver one event no earlier than a same-clock absolute deadline without exposing timers or routes | `TK-ELEMENT-ENTRY-001`, `TK-ELEMENT-OBSERVATION-001`, `TK-DEC-026` | `TOPAL-SYSTEMS-DEADLINE-001`, [deadline-event contract](design/deadline-event.md), [implementation report](step-4-deadline-event.md), checked root, x86 provider and boot adapter, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | same-clock affine lifecycle, early-delivery rejection, typed completion before resume, private HPET comparator/I/O APIC route, generated entry, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| transfer between kernel continuations without exposing registers or stack pointers | `TK-ELEMENT-CONTEXT-001`, `TK-DEC-027` | `TOPAL-SYSTEMS-CONTEXT-001`, [kernel-context contract](design/kernel-context-transfer.md), [implementation report](step-4-kernel-context-transfer.md), checked root, x86 provider, linked worker entry, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine create/transfer/retire/reclaim model, exact caller resumption, private x86 callee-saved state, separate worker stack, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| select multiple cooperative kernel contexts without provider-owned scheduler policy | `TK-ELEMENT-CONTEXT-001`, `TK-DEC-028` | `TOPAL-SYSTEMS-CONTEXT-002`, [cooperative handoff contract](design/cooperative-context-handoff.md), [implementation report](step-4-cooperative-context-handoff.md), checked root, x86 provider, two linked worker entries, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | two disjoint contexts, suspended and retired outcomes, source FIFO selection, exact target resumption, stack reclamation, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| store runnable contexts without provider-owned scheduler policy | `TK-ELEMENT-CONTEXT-001`, `TK-DEC-029` | `TOPAL-SYSTEMS-CONTEXT-003`, [bounded affine runnable-queue contract](design/affine-runnable-queue.md), [implementation report](step-4-affine-runnable-queue.md), checked root, erased x86 root lowering, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | fixed-capacity affine FIFO model, full/empty ownership preservation, exact source selection, provider-boundary inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| preempt one selected kernel context without provider scheduler policy | `TK-ELEMENT-CONTEXT-001`, `TK-ELEMENT-OBSERVATION-001`, `TK-DEC-030` | `TOPAL-SYSTEMS-CONTEXT-004`, [deadline-preemptible context-transfer contract](design/deadline-preemptible-context-transfer.md), [implementation report](step-4-deadline-preemptible-context-transfer.md), checked root, x86 provider, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | architecture-neutral affine model, checked source lifecycle, private interrupted-state lowering, exact dispatcher return, queue re-enqueue, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| block and wake one kernel context without a lost wakeup | `TK-ELEMENT-CONTEXT-001`, `TK-MAP-WAIT-001`, `TK-DEC-031` | `TOPAL-SYSTEMS-CONTEXT-005`, [one-shot kernel-completion contract](design/one-shot-kernel-completion.md), [implementation report](step-4-one-shot-kernel-completion.md), checked root and x86 lowering, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine endpoint model, signal-before-wait persistence, exact blocked-context return, source-owned enqueue, negative tests, structural inspection, and two deterministic pinned-QEMU runs pass |
| enter user mode and complete one Linux syscall round trip without exposing target frames | `TK-ELEMENT-ENTRY-001`, `TK-MAP-ENTRY-001`, `TK-MAP-SYSCALL-001`, `TK-DEC-032` | `TOPAL-SYSTEMS-CONTEXT-006`, [user syscall round-trip contract](design/user-syscall-roundtrip.md), [implementation report](step-4-user-syscall-roundtrip.md), checked root and x86 lowering, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine model and checked lifecycle, private user mappings/selectors/MSRs/stack switch, exact `getpid` return and `exit_group(0)` termination, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| back user execution with source-accounted physical and address-space ownership | `TK-KERNEL-MEMORY-001`, `TK-ELEMENT-ENTRY-001`, `TK-DEC-035`, `TK-DEC-036`, `TK-DEC-037` | `TOPAL-SYSTEMS-FRAMES-001`, `TOPAL-SYSTEMS-CONTEXT-006`, [allocator-backed user-probe contract](design/allocator-backed-user-probe.md), [opaque address-space contract](design/opaque-user-address-space.md), [backing report](step-4-allocator-backed-user-probe.md), [address-space report](step-4-opaque-user-address-space.md), checked root and x86 lowering, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | fresh opaque address-space identity, exact three-frame ownership transfer, distinct preparation/binding/revocation/reclaim, sealed code/stack/leaf-table roles, non-writable code, non-executable stack, inaccessible translation metadata, exact extent return/release, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass; general mapping edits and multiple spaces remain |
| qualify reusable affine user mappings without exposing addresses or page-table formats | `TK-KERNEL-MEMORY-001`, `TK-DEC-038`, `TK-DEC-039`, `TK-DEC-040`, `TK-DEC-041`, `TK-DEC-042`, `TK-DEC-043`, `TK-DEC-044`, `TK-DEC-045`, `TK-DEC-046`, `TK-DEC-047`, `TK-DEC-048` | `TOPAL-SYSTEMS-FRAMES-001`, `topal.systems.memory.user-address-space.verify-mapping/1`, `topal.systems.memory.user-address-space.map/1`, `topal.systems.memory.user-mapping.store-byte/1`, `topal.systems.memory.user-mapping.protect-read-only/1`, `topal.systems.memory.user-mapping.load-byte/1`, `topal.systems.memory.user-mapping.install-entry-trampoline/1`, `topal.systems.memory.user-mapping.verify-entry-trampoline/1`, `topal.systems.memory.user-mapping.permit-execute/1`, `topal.systems.memory.user-mapping.seal-immutable-data/1`, `topal.systems.memory.user-address-space.unmap/1`, [disjoint live-extent contract](design/disjoint-live-frame-extent.md), [sealed mapping contract](design/sealed-user-mapping-round-trip.md), [affine mapping contract](design/affine-user-mapping-lifecycle.md), [sequential-generation contract](design/sequential-user-mapping-generations.md), [permission-tightening contract](design/user-mapping-permission-tightening.md), [executable-promotion contract](design/user-mapping-executable-promotion.md), [image-role contract](design/user-mapping-image-roles.md), [resident-image contract](design/resident-user-image-mappings.md), [readiness-witness contract](design/resident-user-image-readiness.md), [execution-residency contract](design/user-image-execution-residency.md), [constructed-entry contract](design/bounded-constructed-image-entry.md), [closed-transaction report](step-4-sealed-user-mapping-round-trip.md), [affine-lifecycle report](step-4-affine-user-mapping-lifecycle.md), [sequential-generation report](step-4-sequential-user-mapping-generations.md), [permission-tightening report](step-4-user-mapping-permission-tightening.md), [executable-promotion report](step-4-user-mapping-executable-promotion.md), [image-role report](step-4-user-mapping-image-roles.md), [resident-image report](step-4-resident-user-image-mappings.md), [readiness-witness report](step-4-resident-user-image-readiness.md), [execution-residency report](step-4-user-image-execution-residency.md), [constructed-entry report](step-4-constructed-image-entry.md), checked root and x86 lowering, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | exact disjoint extent consumption and return, distinct data byte-access and code entry-payload refinements, two separately allocated fresh mapping identities, writable-authority revocation before verification, W^X-preserving executable code and RO/NX immutable data, concurrent role ownership, initial fetch from the code-role mapping, transfer to the sealed qualification probe, exact readiness/entry/residency witnesses, pairwise backing and probe-range exclusion, data-first teardown, provider-private validation and completion, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass; arbitrary layouts, writable restoration, a third mapping, arbitrary image payloads, general mapping concurrency, and general image loading remain |
| copy kernel data to an untrusted user candidate and recover only the generated fault scope | `TK-ELEMENT-USER-ACCESS-001`, `TK-ELEMENT-FAULT-001`, `TK-MAP-USERCOPY-001`, `TK-DEC-033` | `TOPAL-SYSTEMS-RECOVERY-001`, [fault-contained `copy_to_user` and `uname` contract](design/user-copy-to-uname.md), [implementation report](step-4-user-copy-to-uname.md), checked root, x86 provider and boot adapter, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine scope model, exact-entry source checking, private generated page-fault recovery, Linux `uname` success and `-EFAULT`, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| copy and validate a user layout without exposing partial kernel staging | `TK-ELEMENT-USER-ACCESS-001`, `TK-ELEMENT-FAULT-001`, `TK-MAP-USERCOPY-001`, `TK-DEC-034` | `TOPAL-SYSTEMS-RECOVERY-001`, [fault-contained `copy_from_user` and `nanosleep` contract](design/user-copy-from-timespec.md), [implementation report](step-4-user-copy-from-timespec.md), checked root, x86 provider and boot adapter, and [QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json) | affine scope and complete-value model, partial-value rejection, exact-entry source checking, direction-specific generated page-fault recovery, Linux zero-duration `nanosleep` success and `-EFAULT`, structural inspection, negative tests, and two deterministic pinned-QEMU runs pass |
| keep new elements portable in meaning | `TK-ELEMENT-*` | [cross-architecture pressure test](design/architecture-pressure-test.md), `TOPAL-SYSTEMS-QUALIFY-001`, [x86-64 provider plan](step-4-x86-provider-plan.md), [generated provider object](step-4-x86-provider-object.md), [linked artifact](step-4-linked-artifact.md), [pinned-QEMU gate](step-4-toolchain-gate.md) | common semantics remain separate from typed x86 objects and artifact publication; the initial x86 provider path has physical evidence |
| fit Linux responsibilities to Topal | `TK-KERNEL-*`, `TK-MAP-*` | [kernel architecture](design/topal-kernel-architecture.md), [mapping ledger](design/linux-to-topal-mapping.md) | module designs, implementations, and conformance evidence |
| map Linux driver frameworks | `LK-DRIVER-*` | [driver map](knowledge/common/driver-frameworks.md) | Topal protocol mappings and subsystem conformance ledgers |
| support DTS/DTB | `LK-DT-*` | [Device Tree map](knowledge/common/device-tree.md), [binding inventory](inventory/7.2.9/x86_64/devicetree-bindings.json) | parser/binding tests and selected platform boot |
| describe board support | `LK-X64-BOARD-001`, `LK-X64-BOARD-002`, `LK-X64-DISCOVERY-001` | [platform map](knowledge/x86_64/platform-and-board.md), [machine manifest](labs/qemu/x86_64/manifest.json) | [two-kernel runtime discovery evidence](labs/qemu/x86_64/results/verification.json) |
| run standard applications | `LK-SCOPE-005` | acceptance ladder | pinned rootfs, libc/application corpus, result records |
| run Docker and Podman | `LK-UAPI-CONTAINER-001` | [UAPI map](knowledge/common/linux-uapi.md) | Step 3 rootful smoke [evidence](labs/qemu/x86_64/results/verification.json); full rootful/rootless profiles remain |
| run VMs on the Topal kernel | `LK-UAPI-KVM-001` | [UAPI map](knowledge/common/linux-uapi.md) | QEMU TCG tests, KVM API tests, optional nested profile |

## Step 1 exit criteria

- Linux release and architecture scope are immutable and reproducible.
- “All public interfaces” has an explicit inclusion rule and status model.
- The native syscall table, installed UAPI headers, documented ABI entries,
  direct ioctl definitions, vDSO exports, and Device Tree bindings have
  reproducible counts, identities, locations, and aggregate digests.
- UAPI, kernel, driver, Device Tree, ABI, board, container, and virtualization
  research have stable identities, reviewed boundaries, and primary source
  families.
- The no-source-assembly direction has semantic design criteria and an open
  decision register rather than premature syntax.
- No protected Topal meaning or Linux-derived code has been introduced.
- Remaining semantic enumeration and runtime qualification are explicitly
  carried into later steps rather than hidden behind source-file counts.

## Step 2 proposal exit criteria

- The internal kernel architecture preserves Linux external contracts without
  mechanically translating Linux internal C design.
- Every Linux subsystem family from Step 1 has a stable Topal responsibility
  mapping and a repeatable future-baseline update procedure.
- Existing Topal foundations and mandatory kernel gaps are distinguished.
- Proposed new elements are limited to machine-irreducible semantics; ordinary
  kernel policies and subsystems remain libraries/modules.
- x86-64, AArch64, and RISC-V pressure-test the abstraction while only x86-64
  receives implementation scope.
- Protected Topal meaning remains unchanged until explicit human approval and
  authority-ordered propagation.

## Current Step 4 image refinement

`TK-DEC-049` traces through the
[constructed-image data-observation contract](design/bounded-constructed-image-data-observation.md),
the checked root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-constructed-image-data-observation.md), and
[pinned-QEMU evidence](labs/qemu/x86_64/results/topal-toolchain-gate.json).
The evidence must prove a provider-sealed read-only sentinel observation before
fail-closed transfer, an exact post-return witness, and deterministic physical
execution without introducing source-visible addresses or general loader
policy.

## Next Step 4 syscall refinement

`TK-DEC-050` traces through the
[constructed-image syscall contract](design/bounded-constructed-image-syscall.md),
the checked root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-constructed-image-syscall.md),
and pinned-QEMU evidence. The evidence must prove a Linux-compatible `getpid` transition from
the sealed constructed-code continuation after its data observation and the
existing probe, PID-one validation before terminal `exit_group(0)`, an exact
post-return witness, and deterministic physical execution without exposing
general syscall selection to source.

## Next Step 4 constructed-output refinement

`TK-DEC-051` traces through the
[constructed-image `uname` contract](design/bounded-constructed-image-uname.md),
the checked root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-constructed-image-uname.md),
and pinned-QEMU evidence. The evidence must prove a sealed `uname` output into bounded writable
user-stack storage, complete transfer and returned `Linux` validation,
balanced disposal before the existing constructed identity call, an exact
post-return witness, and deterministic physical execution without exposing
general user candidates or layouts to source.

## Next Step 4 constructed-input refinement

`TK-DEC-052` traces through the
[constructed-image `nanosleep` contract](design/bounded-constructed-image-nanosleep.md),
the checked root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-constructed-image-nanosleep.md),
and pinned-QEMU evidence. The evidence must prove sealed zero-duration input from bounded
writable user-stack storage, complete transfer and value validation, balanced
disposal before the existing constructed identity call, an exact post-return
witness, and deterministic physical execution without exposing general user
candidates, values, or layouts to source.

## Next Step 4 process-entry refinement

`TK-DEC-053` traces through the
[bounded initial-stack contract](design/bounded-initial-user-stack.md), the
checked root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-initial-user-stack.md), and
pinned-QEMU evidence. The evidence must prove aligned bounded initialization, exact
`argc`/`argv`/empty-environment/`AT_NULL` contents, entry at that block,
constructed validation and consumption, balanced continuation stacks, an exact
post-return witness, and deterministic physical execution without exposing
general loader stack policy to source.

## Next Step 4 auxiliary-vector refinement

`TK-DEC-054` traces through the
[bounded initial `AT_PAGESZ` contract](design/bounded-initial-auxv-pagesz.md),
the checked root, x86-64 provider lowering, structural tests, the implementation
and [qualification report](step-4-initial-auxv-pagesz.md), and pinned-QEMU
evidence. The evidence must prove
one provider-consistent Linux base-page-size entry before `AT_NULL`, exact
constructed validation and bounded consumption, an ordered post-return
witness, and deterministic physical execution without exposing general
auxiliary-vector or loader policy to source.

## Next Step 4 executable-entry auxiliary-vector refinement

`TK-DEC-055` traces through the
[bounded initial `AT_ENTRY` contract](design/bounded-initial-auxv-entry.md), the
checked root, x86-64 provider lowering, structural tests, the implementation
and [qualification report](step-4-initial-auxv-entry.md), and pinned-QEMU
evidence. The evidence must prove
one sealed executable-entry fact after `AT_PAGESZ` and before `AT_NULL`, exact
agreement with the selected initial instruction, constructed validation and
bounded consumption, an ordered post-return witness, and deterministic
physical execution without exposing addresses or general ELF-loader policy to
source.

## Next Step 4 executable-name auxiliary-vector refinement

`TK-DEC-056` traces through the
[bounded initial `AT_EXECFN` contract](design/bounded-initial-auxv-execfn.md),
the checked root, x86-64 provider lowering, structural tests, the implementation
and [qualification report](step-4-initial-auxv-execfn.md), and pinned-QEMU
evidence. The evidence must prove
one sealed executable-name fact after `AT_ENTRY` and before `AT_NULL`, exact
pointer identity with the bounded `argv[0]` string, constructed validation and
consumption, an ordered post-return witness, and deterministic physical
execution without exposing addresses or general loader policy to source.

## Next Step 4 credential auxiliary-vector refinement

`TK-DEC-057` traces through the
[bounded initial credential contract](design/bounded-initial-auxv-credentials.md),
the checked root, x86-64 provider lowering, structural tests, the implementation
and [qualification report](step-4-initial-auxv-credentials.md), and pinned-QEMU
evidence. The evidence must prove an
ordered real/effective user/group fact group matching the sealed process,
constructed validation and bounded consumption, an ordered post-return
witness, and deterministic physical execution without exposing credential
encoding or transition policy to source.

## Next Step 4 bounded ELF identity refinement

`TK-DEC-058` traces through the
[bounded ELF identity contract](design/bounded-elf-identity.md), the checked
root, x86-64 provider lowering, structural tests, the
[implementation and qualification report](step-4-bounded-elf-identity.md), and pinned-QEMU
evidence. The evidence must prove a
valid resident target ELF header and program-header table, agreement among the
ELF entry, selected first instruction, and `AT_PHDR`/`AT_PHENT`/`AT_PHNUM`
facts, exact constructed validation and stack consumption, an ordered witness,
and deterministic physical execution without admitting a general loader.

# Topal Linux-compatible kernel project

This tree owns the research, compatibility records, design proposals, test
labs, and future kernel source for an x86-64 kernel implemented primarily in
Topal. The intended result is a new kernel with Topal-native internals and the
userspace-visible behavior of one pinned Linux release. It is not a
line-by-line translation of the Linux kernel.

The first implementation target is native 64-bit x86 userspace on an x86-64
kernel. Other architectures inform the abstraction of new Topal elements, but
they do not receive code generation, a boot target, or conformance commitments
in this phase.

## Current phase

The foundation and Step 1 research map define scope and Linux source coverage.
Step 2 adds an approved [Topal-native kernel design](step-2-report.md), Linux
responsibility mapping, and cross-architecture systems-element model. The
approved meaning is adopted in the authoritative Topal systems profile,
requirements, and formal rules. Step 3 adds a verified
[x86-64 Linux reference VM](step-3-report.md), including the direct replacement
kernel path and Docker/Podman smoke controls. The first Step 4 increment adds
the [checked systems-root semantic foundation](step-4-semantic-foundation.md)
and a model-only x86-64 QEMU kernel target. The second increment adds
[bounded bootstrap storage](step-4-bootstrap-storage.md) to that root and its
architecture-independent reference model. The third increment adds the
[sealed x86-64 provider plan](step-4-x86-provider-plan.md). The fourth adds a
[generated and structurally inspected provider object](step-4-x86-provider-object.md).
The fifth adds a [linked freestanding payload and atomic artifact set](step-4-linked-artifact.md).
The sixth adds the [generated Linux x86 boot adapter](step-4-x86-boot-adapter.md).
The seventh [passes the initial toolchain gate under pinned QEMU](step-4-toolchain-gate.md):
entry, polling serial output, debug-break recovery, checked kernel-owned-memory
use, and the fatal halt execute without a host runtime. The ordinary compiler
now exposes the [qualified systems-artifact publication path](step-4-compiler-publication.md)
without routing it through the Linux-process pipeline. Boot packaging remains
a separate qualified adapter, and the pinned-QEMU evidence consumes the
ordinary compiler output before packaging.

An earlier Step 4 kernel-foundation slice uses the approved
[portable boot-memory description contract](design/boot-memory-description.md)
to refine the provider-private Linux x86 handoff into architecture-neutral
physical-range capabilities before ordinary kernel initialization continues.
Its [checked model and initial x86 E820 provider](step-4-boot-memory.md) are
implemented and qualified by reproducible pinned-QEMU evidence.

The following Step 4 slice adds the approved
[affine physical-frame allocator](design/physical-frame-allocation.md). Its
[implementation and qualification report](step-4-frame-allocation.md) records
the architecture-neutral ownership model, sealed one-frame x86 provider
lowering, structural artifact checks, and two deterministic pinned-QEMU runs.

The later [bounded multi-frame refinement](design/bounded-multi-frame-extent.md)
qualifies one static contiguous three-frame request while retaining a single
indivisible affine extent. Its
[implementation and qualification report](step-4-bounded-multi-frame-extent.md)
records exact request-shape checking, complete-extent E820 selection, and two
deterministic pinned-QEMU runs without admitting dynamic or simultaneous
extents.

The next Step 4 slice adds the approved
[opaque kernel mapping](design/kernel-mapping.md). Its
[implementation and qualification report](step-4-kernel-mapping.md) records
affine map/access/unmap semantics, an adopted x86-64 bootstrap identity
mapping, real indirect frame access, structural checks, and two deterministic
pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[translation-space lifecycle](design/translation-space.md). Its
[implementation and qualification report](step-4-translation-space.md) records
the architecture-independent affine builder/commit/activation model, a
replacement x86-64 four-level root, structural CR3 evidence, and two
deterministic pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[active translation-edit lifecycle](design/active-translation-edit.md). Its
[implementation and qualification report](step-4-active-translation-edit.md)
records exclusive provisional map/unmap transactions, an opaque x86-64 4 KiB
mapping window, commit-time publication and invalidation, and two deterministic
pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[local-interrupt critical scope](design/local-interrupt-critical-scope.md). Its
[implementation and qualification report](step-4-local-interrupt-critical-scope.md)
records the affine context refinement, exact prior-state restoration model,
private x86 flags lowering, structural instruction checks, and two
deterministic pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[atomic-word location lifecycle](design/atomic-word-location.md). Its
[implementation and qualification report](step-4-atomic-word-location.md)
records the affine region transition, modification-order model, private x86
locked compare/exchange and acquire load, structural instruction checks, and
two deterministic pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[local-notification interrupt lifecycle](design/local-notification-interrupt.md).
Its [implementation and qualification report](step-4-local-notification-interrupt.md)
records the affine send/wait observation model, mandatory typed-entry
completion, private x86-64 local-APIC delivery and EOI, generated interrupt
gate and state preservation, structural inspection, and two deterministic
pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[monotonic-clock observation](design/monotonic-clock-observation.md). Its
[implementation and qualification report](step-4-monotonic-clock-observation.md)
records the same-clock nondecreasing observation model, private Q35 HPET
provider, initial and replacement MMIO mappings, structural inspection, and
two deterministic pinned-QEMU runs.

An earlier Step 4 slice adds the approved
[one-shot deadline event](design/deadline-event.md). Its
[implementation and qualification report](step-4-deadline-event.md) records
same-clock absolute-deadline semantics, affine arm/wait/completion, a private
Q35 HPET comparator and I/O APIC route, generated typed entry, structural
inspection, and two deterministic pinned-QEMU runs.

The latest Step 4 slice adds the approved
[kernel-context transfer](design/kernel-context-transfer.md). Its
[implementation and qualification report](step-4-kernel-context-transfer.md)
records affine continuation and stack ownership, terminal worker retirement,
exact suspended-caller resumption, private x86-64 callee-saved state, a
separate worker stack, structural inspection, and two deterministic pinned-QEMU
runs.

The current Step 4 slice adds the approved
[cooperative context handoff](design/cooperative-context-handoff.md). Its
[implementation and qualification report](step-4-cooperative-context-handoff.md)
records two affine worker contexts, source-level FIFO selection, one suspended
handoff, two terminal retirements and reclamations, a symmetric private x86-64
stack switch, structural inspection, and two deterministic pinned-QEMU runs.

The latest Step 4 slice adds the approved
[bounded affine runnable queue](design/affine-runnable-queue.md). Its
[implementation and qualification report](step-4-affine-runnable-queue.md)
records a capacity-two source-owned FIFO, full/empty ownership semantics,
provider-free queue lowering, a 99-transition artifact trace, and two
deterministic pinned-QEMU runs with unchanged kernel and serial hashes.

The current Step 4 slice adds the approved
[deadline-preemptible context transfer](design/deadline-preemptible-context-transfer.md).
Its [implementation and qualification report](step-4-deadline-preemptible-context-transfer.md)
records one deadline-bound worker, exact interrupt-continuation preservation,
return to the source dispatcher, affine queue re-enqueue, later restoration,
and two deterministic pinned-QEMU runs.

The latest Step 4 slice adds the approved
[one-shot affine kernel completion](design/one-shot-kernel-completion.md). Its
[implementation and qualification report](step-4-one-shot-kernel-completion.md)
records persistent signal-first readiness, an exact block-first dispatcher,
source-owned wakeup and FIFO selection, three private x86-64 context records,
and two deterministic pinned-QEMU runs.

The current Step 4 slice adds the approved
[initial user syscall round trip](design/user-syscall-roundtrip.md). Its
[implementation and qualification report](step-4-user-syscall-roundtrip.md)
records one affine PID-one user execution, typed Linux syscall entry,
`getpid` return, `exit_group(0)` termination, private x86-64 ring-3 and
`SYSCALL`/`SYSRETQ` lowering, structural inspection, and two deterministic
pinned-QEMU runs.

The current Step 4 slice adds the approved systems-only
[fault-contained `copy_to_user` and `uname`](design/user-copy-to-uname.md)
contract. It ties transfer authority to the exact syscall entry, preserves
untrusted user candidates as non-references, reports explicit prefix progress,
and qualifies successful and recovered-fault `uname` paths without introducing
file-descriptor or core-language exception semantics. Its
[implementation and qualification report](step-4-user-copy-to-uname.md)
records the checked lifecycle, generated recovery entry, and two deterministic
pinned-QEMU runs.

The latest Step 4 slice adds the approved systems-only
[fault-contained `copy_from_user` and zero-duration `nanosleep`](design/user-copy-from-timespec.md)
contract. It keeps input staging private until a complete selected layout is
validated and exposes no partial value on fault. Its
[implementation and qualification report](step-4-user-copy-from-timespec.md)
records the checked lifecycle, direction-specific generated recovery, and two
deterministic pinned-QEMU runs without changing the core language.

The allocator-backed user-probe refinement then applies the approved
[user-backing ownership contract](design/allocator-backed-user-probe.md).
Its [implementation and qualification report](step-4-allocator-backed-user-probe.md)
records exact three-frame ownership transfer into user execution, sealed x86-64
code/stack/translation roles, complete mapping revocation, exact extent return
and release, and two deterministic pinned-QEMU runs. It changes only the
systems profile and does not change the core language.

The next systems-only refinement separates that backing into an
[opaque user-address-space lifecycle](design/opaque-user-address-space.md).
Its [implementation and qualification report](step-4-opaque-user-address-space.md)
records distinct preparation, context binding, revocation, reclaim, and release
transitions; split x86-64 provider operations; and two deterministic pinned-
QEMU runs without changing core-language meaning.

The following systems-only refinement adds
[affine executable promotion](design/user-mapping-executable-promotion.md)
after read-only verification. Its
[implementation and qualification report](step-4-user-mapping-executable-promotion.md)
records W^X-preserving RO/NX-to-RO/X authority, provider-private completion,
structural inspection, and two deterministic pinned-QEMU runs.

The latest systems-only refinement qualifies a
[bounded disjoint live-frame extent](design/disjoint-live-frame-extent.md)
while the prepared address space retains its backing. Its
[implementation and qualification report](step-4-disjoint-live-frame-extent.md)
records distinct simultaneous ownership, provider-private interval exclusion,
exact release before user binding, structural x86-64 inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The current systems-only refinement consumes that disjoint frame in one
[sealed user-mapping round trip](design/sealed-user-mapping-round-trip.md).
Its [implementation and qualification report](step-4-sealed-user-mapping-round-trip.md)
records opaque map/access/revoke ownership, private x86-64 RW/NX leaf
publication and invalidation, exact extent return, structural inspection, and
two deterministic pinned-QEMU runs without changing core-language meaning.

The latest systems-only refinement splits that transaction into a
[bounded affine user-mapping lifecycle](design/affine-user-mapping-lifecycle.md).
Its [implementation and qualification report](step-4-affine-user-mapping-lifecycle.md)
records distinct map, store, load, and unmap refinements, persistent opaque
mapping ownership, exact frame return, structural x86-64 inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The next systems-only refinement permits
[two sequential mapping generations](design/sequential-user-mapping-generations.md)
over that exact returned frame. Its
[implementation and qualification report](step-4-sequential-user-mapping-generations.md)
records fresh opaque identities, complete inter-generation revocation,
provider-state reset, exact final frame return, and two deterministic pinned-
QEMU runs without changing core-language meaning.

The current systems-only refinement adds
[affine user-mapping permission tightening](design/user-mapping-permission-tightening.md)
to both generations. Its
[implementation and qualification report](step-4-user-mapping-permission-tightening.md)
records the stored-to-protected ownership transition, provider-private x86-64
RW-to-RO publication and invalidation, structural inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The following systems-only refinement assigns
[sealed immutable code and data roles](design/user-mapping-image-roles.md) to
the two verified generations. Its
[implementation and qualification report](step-4-user-mapping-image-roles.md)
records disjoint RO/X code and RO/NX data refinements, role-specific x86-64
validation, structural inspection, and two deterministic pinned-QEMU runs
without changing core-language meaning.

The next systems-only refinement keeps those roles in
[bounded resident code and data mappings](design/resident-user-image-mappings.md)
during construction. Its
[implementation and qualification report](step-4-resident-user-image-mappings.md)
records two separately allocated disjoint frames, concurrent RO/X and RO/NX
ownership, data-first teardown, structural x86-64 inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The latest systems-only refinement requires a
[bounded constructed-image data observation](design/bounded-constructed-image-data-observation.md)
before the sealed code payload transfers to the existing probe. Its
[implementation and qualification report](step-4-constructed-image-data-observation.md)
records exact payload installation and verification, a fail-closed immutable
sentinel read, ordered post-return witnesses, structural x86-64 inspection,
and two deterministic pinned-QEMU runs without changing core-language meaning.

The following systems-only refinement adds a
[bounded constructed-image syscall](design/bounded-constructed-image-syscall.md).
Its [implementation and qualification report](step-4-constructed-image-syscall.md)
records sealed transfer back into constructed code, direct `getpid` and PID-one
validation, terminal `exit_group(0)`, exact call-site inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The next systems-only refinement adds bounded pointer-bearing output through
[constructed-image `uname`](design/bounded-constructed-image-uname.md). Its
[implementation and qualification report](step-4-constructed-image-uname.md)
records sealed writable-stack staging, complete `copy_to_user`, returned
`Linux` validation, balanced disposal, exact call-site inspection, and two
deterministic pinned-QEMU runs without changing core-language meaning.

The following systems-only refinement adds bounded pointer-bearing input
through [constructed-image `nanosleep`](design/bounded-constructed-image-nanosleep.md).
Its [implementation and qualification report](step-4-constructed-image-nanosleep.md)
records sealed zero-duration stack staging, complete `copy_from_user`, input
validation, balanced disposal, exact call-site inspection, and repeated
pinned-QEMU qualification without changing core-language meaning.

The next systems-only refinement constructs a
[bounded Linux-compatible initial user stack](design/bounded-initial-user-stack.md).
Its [implementation and qualification report](step-4-initial-user-stack.md)
records exact `argc`/`argv`/empty-environment/`AT_NULL` initialization,
constructed validation and bounded consumption, structural inspection, and
two deterministic pinned-QEMU runs without changing core-language meaning.

The following systems-only refinement adds a
[bounded initial `AT_PAGESZ` auxiliary-vector entry](design/bounded-initial-auxv-pagesz.md).
It keeps the tag, value, word layout, and addresses provider-owned while
requiring constructed code to validate the Linux base-page-size fact before
continuing. Its
[implementation and qualification report](step-4-initial-auxv-pagesz.md)
records exact construction, validation, bounded consumption, structural
inspection, and two deterministic pinned-QEMU runs.

The next systems-only refinement binds a
[bounded initial `AT_ENTRY` fact](design/bounded-initial-auxv-entry.md) to the
first instruction selected by the provider. It keeps the tag, address, and
layout provider-owned while requiring constructed code to validate exact
agreement with its own sealed entry point. Its
[implementation and qualification report](step-4-initial-auxv-entry.md)
records exact construction, correspondence validation, bounded consumption,
structural inspection, and two deterministic pinned-QEMU runs.

The following systems-only refinement adds a
[bounded initial `AT_EXECFN` fact](design/bounded-initial-auxv-execfn.md) that
aliases the already validated `argv[0]` string. It keeps the tag, pointer, and
layout provider-owned while requiring constructed code to prove exact pointer
identity and in-block string validation. Its
[implementation and qualification report](step-4-initial-auxv-execfn.md)
records exact construction, alias validation, bounded consumption, structural
inspection, and two deterministic pinned-QEMU runs.

The next systems-only refinement adds the
[bounded initial credential fact group](design/bounded-initial-auxv-credentials.md)
for the sealed PID-one root profile. It keeps tag and credential encodings
provider-owned while requiring coherent real/effective user/group validation.
Its [implementation and qualification report](step-4-initial-auxv-credentials.md)
records exact construction, ordered validation, bounded consumption, structural
inspection, and two deterministic pinned-QEMU runs.

The following systems-only refinement adds a
[bounded ELF identity and program-header facts](design/bounded-elf-identity.md)
inside the existing code-role mapping. It binds the selected entry and
`AT_PHDR`/`AT_PHENT`/`AT_PHNUM` facts to one provider-sealed resident ELF64
header and table without admitting arbitrary ELF input or general loading. Its
[implementation and qualification report](step-4-bounded-elf-identity.md)
records exact image installation, entry and table correspondence, bounded
stack consumption, structural inspection, and two deterministic QEMU runs.

The next systems-only refinement adds a
[provider-neutral initial boot resource](design/initial-boot-resource.md).
Its [implementation and qualification report](step-4-initial-boot-resource.md)
records deterministic CPIO publication, opaque Linux ramdisk handoff,
reservation and allocator exclusion, complete identity validation, and two
deterministic pinned-QEMU runs without exposing transport details to source.

The following systems-only refinement performs a
[bounded external ELF load](design/bounded-external-elf-load.md) from that
resource into the existing allocator-backed code-role mapping. Its
[implementation and qualification report](step-4-bounded-external-elf-load.md)
records exact member discovery, resource-to-mapping population, W^X
finalization, structural inspection, and two deterministic pinned-QEMU runs.

The next systems-only refinement makes the
[external ELF payload authoritative](design/external-elf-payload-authority.md)
at runtime. Its
[implementation and qualification report](step-4-external-elf-payload-authority.md)
records sealed metadata validation, same-resource installation and post-copy
comparison, W^X publication, and two deterministic pinned-QEMU runs without a
provider-embedded executable equality oracle.

The next systems-only refinement adds a distinct
[external data segment and explicit BSS](design/bounded-external-data-bss.md).
Its [implementation and qualification report](step-4-bounded-external-data-bss.md)
records resource-backed initialized-data copying, complete BSS zeroing,
RO/NX publication, coherent three-entry program-header facts, and two
deterministic pinned-QEMU runs.

The following systems-only refinement derives the
[executable extent from runtime ELF metadata](design/runtime-derived-executable-extent.md).
Its [implementation and qualification report](step-4-runtime-derived-executable-extent.md)
records relational size validation, bounded copy and post-protection compare
loops, removal of the build-time unrolled extent, and two deterministic
pinned-QEMU runs.

The next systems-only refinement derives the
[initialized-data and BSS extents from runtime ELF metadata](design/runtime-derived-data-extent.md).
Its [implementation and qualification report](step-4-runtime-derived-data-extent.md)
records relational file/memory-size validation, complete bounded copy and
explicit-zero loops, post-protection comparison and zero verification, and two
deterministic pinned-QEMU runs.

The following systems-only refinement derives the
[initial instruction pointer from runtime ELF metadata](design/runtime-derived-entry.md).
Its [implementation and qualification report](step-4-runtime-derived-entry.md)
records executable-extent containment, one retained selected entry, consistent
`AT_ENTRY` and ring-3 entry use, resident-header comparison, and two
deterministic pinned-QEMU runs.

The next systems-only refinement makes the
[initial user ELF a canonical published artifact](design/published-initial-user-image.md).
Its [implementation and qualification report](step-4-published-initial-user-image.md)
records atomic publication, artifact-provenance binding, exact-byte boot
packaging, tamper rejection, and two deterministic pinned-QEMU runs.

The following systems-only refinement accepts one
[caller-supplied bounded initial ELF](design/supplied-initial-user-image.md).
Its [implementation and qualification report](step-4-supplied-initial-user-image.md)
records prepublication envelope validation, systems-only CLI selection,
content-based provenance, a payload distinct from the generated default, and
two deterministic pinned-QEMU runs.

The next systems-only refinement gives the initial ELF a typed
[writable non-executable data role](design/writable-user-data.md). Its
[implementation and qualification report](step-4-writable-user-data.md)
records verified RO/NX-to-RW/NX promotion, retained W^X, an in-program BSS
write/read proof, and two deterministic pinned-QEMU runs.

The following systems-only refinement performs one bounded
[initial-process standard-output write](design/bounded-constructed-image-write.md).
Its [implementation and qualification report](step-4-constructed-image-write.md)
records complete kernel staging before output, exact Linux x86-64 return
behavior, user-stack restoration, and two deterministic pinned-QEMU runs.

The next systems-only refinement performs one bounded
[initial-process standard-input read](design/bounded-constructed-image-read.md).
Its [implementation and qualification report](step-4-constructed-image-read.md)
records deterministic serial injection, provider-private staging, exact
copy-to-user and return behavior, byte validation, and two deterministic
pinned-QEMU runs.

The following systems-only refinement generalizes the
[bounded standard-output write](design/bounded-general-stdout-write.md) to
caller-selected one-to-256-byte extents in known readable mappings. Its
[implementation and qualification report](step-4-bounded-general-stdout-write.md)
records payload-neutral dispatch, complete private staging, Linux-compatible
argument errors, structural inspection, and two deterministic pinned-QEMU runs.

The next systems-only refinement generalizes the
[bounded standard-input read](design/bounded-general-stdin-read.md) to caller-
selected one-to-256-byte extents in known writable mappings. Its
[implementation and qualification report](step-4-bounded-general-stdin-read.md)
records input-value-neutral exact-fill reception, complete private staging
before user publication, Linux-compatible argument errors, structural
inspection, and two deterministic pinned-QEMU runs.

The following systems-only refinement adds a
[bounded initial-process program break](design/bounded-initial-brk.md) within
the existing writable data page. Its
[implementation and qualification report](step-4-bounded-initial-brk.md)
records runtime-ELF-derived private state, complete future-heap zeroing,
query/grow/write-read/shrink/failure behavior, teardown clearing, structural
inspection, and two deterministic pinned-QEMU runs.

The current systems-only driver refinement completes
[bounded virtio queue activation](design/bounded-virtio-queue-activation.md).
Its [implementation and qualification report](step-4-bounded-virtio-queue-activation.md)
records exact queue-enable and `DRIVER_OK` writes and readbacks, reuse of the
generic PCI-region access abstraction, final-action isolation from earlier
interrupt-sensitive tests, structural inspection, and two deterministic
pinned-QEMU runs.

The following systems-only driver refinement performs one
[bounded virtio queue notification](design/bounded-virtio-queue-notification.md).
Its [implementation and qualification report](step-4-bounded-virtio-queue-notification.md)
records source-owned queue-zero selection and notify-offset arithmetic, reuse
of generic PCI-region access, an exact event write with no synthetic readback,
structural inspection, and two deterministic pinned-QEMU runs.

The current systems-only entropy refinement binds the generation-three
generator and reserved first output into a
[reusable process `getrandom` service](design/reusable-process-getrandom.md).
Its [implementation and qualification report](step-4-reusable-process-getrandom.md)
records two consecutive successful calls, fresh generation after the reserved
output, non-consuming failures, complete key/output/service erasure, structural
inspection, and two deterministic pinned-QEMU runs.

The current full-output refinement raises both the initial reservation and the
reusable service to the complete
[32-byte provider output](design/full-output-process-getrandom.md). Its
[implementation and qualification report](step-4-full-output-process-getrandom.md)
records the existing two 16-byte successes followed by a fresh 32-byte call,
complete four-qword observation, non-consuming failures, full-output erasure,
structural inspection, and two deterministic pinned-QEMU runs.

The latest large-request refinement raises the explicit reusable-service bound
to [512 bytes and 16 outputs](design/bounded-large-request-getrandom.md) while
retaining whole-range validation and exact completion in the current
signal-free process profile. Its
[implementation and qualification report](step-4-bounded-large-request-getrandom.md)
records per-output key replacement and erasure, one-call accounting,
structural inspection, and two deterministic pinned-QEMU runs.

The following systems-only userspace refinement introduces
[bounded close state for the three initial descriptors](design/initial-descriptor-close.md).
Its [implementation and qualification report](step-4-initial-descriptor-close.md)
records successful `close(0)`, shared `read`/`write` availability checks,
repeated-close and post-close `EBADF`, terminal state clearing, structural
inspection, and two deterministic pinned-QEMU runs.

The next systems-only userspace refinement adds
[bounded duplication within the three initial descriptor slots](design/initial-descriptor-duplication.md).
Its [implementation and qualification report](step-4-initial-descriptor-duplication.md)
records fixed open-description identities and modes, lowest-free `dup`, atomic
`dup2` replacement, independent alias close, exact `EBADF`/`EMFILE`, terminal
mapping clearing, structural inspection, and two deterministic pinned-QEMU
runs.

The following systems-only userspace refinement adds
[bounded initial descriptor metadata](design/initial-descriptor-fcntl.md).
Its [implementation and qualification report](step-4-initial-descriptor-fcntl.md)
records `F_GETFD`, `F_SETFD`, and `F_GETFL`, separate per-slot `FD_CLOEXEC`,
shared access modes, duplication flag clearing, lookup-before-command error
precedence, terminal erasure, structural inspection, and two deterministic
pinned-QEMU runs.

The next systems-only userspace refinement adds
[bounded native metadata for the initial console descriptions](design/initial-descriptor-fstat.md).
Its [implementation and qualification report](step-4-initial-descriptor-fstat.md)
records the exact x86-64 144-byte `struct stat`, shared-object identity across
aliases, lookup-before-pointer error precedence, complete writable-range
validation, private staging, fault recovery, closed-slot `EBADF`, structural
inspection, and two deterministic pinned-QEMU runs.

The following systems-only userspace refinement adds
[bounded `TCGETS` terminal discovery](design/initial-console-tcgets.md).
Its [implementation and qualification report](step-4-initial-console-tcgets.md)
records the exact native 36-byte raw `termios`, shared input/output attributes,
descriptor/request/pointer error precedence, complete writable-range staging
and fault recovery, exact `EBADF`/`ENOTTY`/`EFAULT`, structural inspection, and
two deterministic pinned-QEMU runs.

The next systems-only userspace refinement adds
[bounded `TIOCGWINSZ` console geometry](design/initial-console-winsize.md).
Its [implementation and qualification report](step-4-initial-console-winsize.md)
records the exact native eight-byte `struct winsize`, truthful zero-valued
unspecified geometry, shared input/output identity, descriptor/request/pointer
error precedence, complete writable-range staging and fault recovery, exact
`EBADF`/`ENOTTY`/`EFAULT`, structural inspection, and two deterministic
pinned-QEMU runs.

The following systems-only userspace refinement adds
[bounded inherited-console `lseek` classification](design/initial-console-lseek.md).
Its [implementation and qualification report](step-4-initial-console-lseek.md)
records stream non-seekability without position state, descriptor-before-mode
precedence, exact `EBADF`/`EINVAL`/`ESPIPE`, input/output and closed-slot
probes, structural inspection, and two deterministic pinned-QEMU runs.

The next systems-only image refinement adds a
[bounded multipage initial executable](design/bounded-multipage-initial-executable.md).
Its [implementation and qualification report](step-4-bounded-multipage-initial-executable.md)
records one affine two-frame code extent, adjacent x86-64 leaves, complete
two-page copy and verification, extent-wide W^X transitions, a second-page
entry and terminal path, atomic whole-extent return, structural inspection,
and two deterministic pinned-QEMU runs.

The following systems-only compiler refinement adds a
[bounded static ELF load plan](design/bounded-static-elf-load-plan.md). Its
[implementation and qualification report](step-4-bounded-static-elf-load-plan.md)
records complete fail-closed ELF64 and program-header validation, immutable
R/RX/R/RW interval planning for the pinned 503-page BusyBox shape, malformed-
input coverage, deliberate runtime nonselection, and two unchanged pinned-QEMU
regression runs.

The next systems-only compiler refinement adds a
[bounded static ELF page realization](design/bounded-static-elf-page-realization.md).
Its [implementation and qualification report](step-4-bounded-static-elf-page-realization.md)
records one future affine 503-frame request, exact ordered page/frame facts,
whole-span zeroing before four copies, W^X-safe entry roles with staged RELRO,
malformed inherited-plan rejection, and two unchanged pinned-QEMU runs.

The following systems-only physical refinement adds the
[bounded static ELF frame extent](design/bounded-static-elf-frame-extent.md).
Its [implementation and qualification report](step-4-bounded-static-elf-frame-extent.md)
records exact 503-frame admission, one opaque indivisible extent, checked E820
whole-range and reservation validation, whole release, structural inspection,
and two deterministic pinned-QEMU runs.

The next systems-only physical refinement adds
[bounded static ELF mapping resources](design/bounded-static-elf-mapping-resources.md).
Its [implementation and qualification report](step-4-bounded-static-elf-mapping-resources.md)
records simultaneous disjoint 503-frame payload and one-frame metadata
ownership, explicit whole-payload provider exclusion, metadata-then-payload
rollback, structural rejection, and two deterministic pinned-QEMU runs.

The latest systems-only execution refinement adds
[bounded static ELF process entry](design/bounded-static-elf-process-entry.md).
Its [implementation and qualification report](step-4-bounded-static-elf-process-entry.md)
records one affine randomized-to-completed transition, real x86-64 CPL3 entry,
native `exit_group(0)` return, exact kernel-state restoration, provider-private
Accessed/Dirty normalization, a qualification-only inverse, and two
deterministic pinned-QEMU runs. The clean-room probe establishes the machine
boundary without yet claiming BusyBox or general userspace compatibility.

The following systems-only startup refinement adds the pinned executable's
[initial `brk(0)` query](design/bounded-static-elf-initial-brk-query.md).
Its [implementation and qualification report](step-4-bounded-static-elf-initial-brk-query.md)
records a provider-derived page-rounded initial break, one real nonterminal
`SYSCALL`/`SYSRETQ` round trip before terminal return, private ordering state,
structural inspection, and two deterministic pinned-QEMU runs without exposing
native syscall or address representation to source.

The next systems-only startup refinement adds one
[bounded initial program-break growth](design/bounded-static-elf-initial-brk-growth.md).
Its [implementation and qualification report](step-4-bounded-static-elf-initial-brk-growth.md)
records an artifact-owned zero-page reserve, one exact RW/NX user mapping, a
first-byte write/read proof, complete terminal revocation and erasure,
structural inspection, and two deterministic pinned-QEMU runs.

The following target-qualified startup refinement installs the pinned static
process's [initial x86-64 FS base](design/bounded-static-elf-initial-x86-fs-base.md).
Its [implementation and qualification report](step-4-bounded-static-elf-initial-x86-fs-base.md)
records one exact thread-pointer selection inside the live anonymous reserve,
an FS-relative access proof, exact pre-entry physical-base restoration, private
state clearing, structural inspection, and duplicate deterministic QEMU
evidence without exposing native controls to source.

The next systems-only startup refinement registers the pinned process's
[initial clear-child-TID word](design/bounded-static-elf-initial-clear-child-tid.md).
Its [qualification report](step-4-bounded-static-elf-initial-clear-child-tid.md)
records exact writable-word authority, sealed TID one, verified terminal
zeroing before address-space restoration, private-state clearing, and duplicate
pinned-QEMU evidence without adding futex or multi-thread policy.

The following systems-only startup refinement registers the pinned process's
[empty robust list](design/bounded-static-elf-initial-empty-robust-list.md).
Its [qualification report](step-4-bounded-static-elf-initial-empty-robust-list.md)
records the exact readable native head, deferred terminal copy and empty-state
validation, private-state clearing, no futex action, and duplicate pinned-QEMU
evidence.

The next systems-only startup refinement preserves the pinned process's
optional [`rseq` probe](design/bounded-static-elf-initial-rseq-fallback.md).
Its [qualification report](step-4-bounded-static-elf-initial-rseq-fallback.md)
records the observed native request, exact side-effect-free `-ENOSYS` result,
unchanged prior startup state, continued terminal dispatch, and repeated
pinned-QEMU evidence without introducing restartable-sequence policy.

The following systems-only startup refinement publishes the pinned process's
[initial stack limit](design/bounded-static-elf-initial-stack-limit-query.md).
Its [qualification report](step-4-bounded-static-elf-initial-stack-limit-query.md)
records the exact query-only `prlimit64`, complete immutable native pair,
temporary-stack non-overlap and erasure, continued terminal dispatch, and
duplicate deterministic QEMU evidence without adding resource enforcement.

The next systems-only startup refinement preserves the pinned process's
optional [`/proc/self/exe` probe](design/bounded-static-elf-initial-readlinkat-fallback.md).
Its [qualification report](step-4-bounded-static-elf-initial-readlinkat-fallback.md)
records exact `readlinkat` selection, side-effect-free `-ENOSYS`, unchanged
startup state, continued terminal dispatch, and duplicate deterministic QEMU
evidence without adding filesystem policy.

The pinned compatibility baseline is [Linux 7.2.9](baseline.md). “Latest” means
the complete observable native x86-64 interface of that immutable baseline
during a qualification run. Updating to a later stable release requires a
reviewed baseline-bump change and a complete interface-drift report.

## Project principles

- Preserve Linux userspace behavior; do not preserve Linux internal structure
  merely for familiarity.
- Target the complete baseline interface, while implementing and qualifying it
  through explicit dependency-ordered increments.
- Introduce typed Topal elements for irreducible machine and kernel operations.
  Source-level inline assembly, instruction strings, clobber lists, and
  programmer-selected physical registers are not part of the design.
- Define portable semantic operations where architectures share meaning, and
  target-qualified operations where they genuinely differ. Do not disguise an
  x86 instruction as a portable abstraction.
- Keep authority, effects, ordering, lifetime, address-space identity, and
  privilege explicit.
- Use generated inventories for breadth and reviewed design records for
  meaning. Absence from a handwritten list never silently removes an upstream
  interface from scope.
- Keep downloaded sources, VM images, package caches, and build products out of
  version control. Commit recipes, manifests, configurations, checksums, and
  results instead.

## Directory plan

```text
linux-kernel/
  baseline.md                 pinned upstream and compatibility contract
  knowledge/                  versioned Linux design knowledge library
    common/                   architecture-independent Linux design
    x86_64/                   native x86-64 ABI and platform facts
  design/                     kernel-specific Topal proposals and decisions
  kernel/                     future Topal kernel source
    common/
    arch/x86_64/
  labs/qemu/x86_64/           reproducible Linux reference/test VM
  tests/                      future differential and conformance tests
  tools/                      future inventory and drift tooling
  traceability.md             stable project identities and evidence links
```

Directories without an artifact are created only when their first reviewed
content exists.

## Phase gates

1. **Knowledge gate:** inventory the baseline UAPI and map Linux architecture,
   driver, board, container, and virtualization designs by stable identity.
2. **Language-design gate:** resolve the open kernel primitive decisions and
   approve any required changes to authoritative Topal design.
3. **Toolchain gate:** emit a bootable x86-64 image that writes to a defined
   console, handles a fault, and uses kernel-owned memory without host syscalls.
4. **Kernel-foundation gate:** boot, memory management, interrupts, time, SMP,
   ACPI/PCI discovery, and selected virtio devices work under pinned QEMU.
5. **Userspace gates:** advance from a static initramfs through dynamic libc and
   the complete baseline interface ledger, with differential Linux evidence.
6. **Container gate:** qualify both selected Docker and Podman profiles.
7. **Virtualization gates:** qualify QEMU software emulation first, then the
   separately scoped KVM host API and optional nested virtualization.

Every gate may use several reviewable pull requests. Its closing pull request
records the completed evidence, remaining gaps, risk assessment, and next gate.

## Navigation

- [Pinned baseline and compatibility contract](baseline.md)
- [Knowledge-library rules](knowledge/README.md)
- [Knowledge catalog](knowledge/catalog.md)
- [Step 1 report](step-1-report.md)
- [Step 2 design proposal](step-2-report.md)
- [Step 3 reference VM report](step-3-report.md)
- [Step 4 systems-root semantic foundation](step-4-semantic-foundation.md)
- [Step 4 bounded bootstrap storage](step-4-bootstrap-storage.md)
- [Step 4 x86-64 provider plan](step-4-x86-provider-plan.md)
- [Step 4 generated x86-64 provider object](step-4-x86-provider-object.md)
- [Step 4 linked freestanding x86-64 artifact](step-4-linked-artifact.md)
- [Step 4 generated Linux x86 boot adapter](step-4-x86-boot-adapter.md)
- [Step 4 pinned-QEMU boot and exception gate](step-4-toolchain-gate.md)
- [Step 4 ordinary compiler systems publication](step-4-compiler-publication.md)
- [Step 4 boot-memory refinement](step-4-boot-memory.md)
- [Step 4 affine physical-frame allocation](step-4-frame-allocation.md)
- [Step 4 bounded multi-frame extent](step-4-bounded-multi-frame-extent.md)
- [Step 4 opaque kernel mapping](step-4-kernel-mapping.md)
- [Step 4 translation-space activation](step-4-translation-space.md)
- [Step 4 active translation edits](step-4-active-translation-edit.md)
- [Step 4 local-interrupt critical scope](step-4-local-interrupt-critical-scope.md)
- [Step 4 atomic-word location](step-4-atomic-word-location.md)
- [Step 4 local-notification interrupt](step-4-local-notification-interrupt.md)
- [Step 4 monotonic-clock observation](step-4-monotonic-clock-observation.md)
- [Step 4 one-shot deadline event](step-4-deadline-event.md)
- [Step 4 kernel-context transfer](step-4-kernel-context-transfer.md)
- [Step 4 cooperative context handoff](step-4-cooperative-context-handoff.md)
- [Step 4 bounded affine runnable queue](step-4-affine-runnable-queue.md)
- [Step 4 deadline-preemptible context transfer](step-4-deadline-preemptible-context-transfer.md)
- [Step 4 one-shot kernel completion](step-4-one-shot-kernel-completion.md)
- [Step 4 initial user syscall round trip](step-4-user-syscall-roundtrip.md)
- [Step 4 fault-contained `copy_to_user` and `uname`](step-4-user-copy-to-uname.md)
- [Step 4 fault-contained `copy_from_user` and zero-duration `nanosleep`](step-4-user-copy-from-timespec.md)
- [Step 4 allocator-backed user probe](step-4-allocator-backed-user-probe.md)
- [Step 4 opaque user-address-space lifecycle](step-4-opaque-user-address-space.md)
- [Step 4 disjoint live-frame extent](step-4-disjoint-live-frame-extent.md)
- [Step 4 sealed user-mapping round trip](step-4-sealed-user-mapping-round-trip.md)
- [Step 4 affine user-mapping lifecycle](step-4-affine-user-mapping-lifecycle.md)
- [Step 4 sequential user-mapping generations](step-4-sequential-user-mapping-generations.md)
- [Step 4 user-mapping permission tightening](step-4-user-mapping-permission-tightening.md)
- [Step 4 user-mapping executable promotion](step-4-user-mapping-executable-promotion.md)
- [Step 4 sealed user-mapping roles](step-4-user-mapping-image-roles.md)
- [Step 4 resident user-image mappings](step-4-resident-user-image-mappings.md)
- [Step 4 resident user-image readiness](step-4-resident-user-image-readiness.md)
- [Step 4 user-image execution residency](step-4-user-image-execution-residency.md)
- [Step 4 bounded constructed-image entry](step-4-constructed-image-entry.md)
- [Step 4 bounded constructed-image data observation](step-4-constructed-image-data-observation.md)
- [Step 4 bounded constructed-image syscall](step-4-constructed-image-syscall.md)
- [Step 4 bounded constructed-image `uname` output](step-4-constructed-image-uname.md)
- [Step 4 bounded constructed-image `nanosleep` input](step-4-constructed-image-nanosleep.md)
- [Step 4 bounded Linux-compatible initial user stack](step-4-initial-user-stack.md)
- [Step 4 bounded initial `AT_PAGESZ` auxiliary vector](step-4-initial-auxv-pagesz.md)
- [Step 4 bounded initial `AT_ENTRY` auxiliary-vector fact](step-4-initial-auxv-entry.md)
- [Step 4 bounded initial `AT_EXECFN` auxiliary-vector fact](step-4-initial-auxv-execfn.md)
- [Step 4 bounded initial credential auxiliary-vector facts](step-4-initial-auxv-credentials.md)
- [Step 4 bounded initial `AT_SECURE` fact](step-4-initial-auxv-secure.md)
- [Step 4 bounded initial `AT_CLKTCK` fact](step-4-initial-auxv-clktck.md)
- [Step 4 bounded initial `AT_PLATFORM` fact](step-4-initial-auxv-platform.md)
- [Step 4 initial scalar identity syscalls](step-4-initial-scalar-identity-syscalls.md)
- [Step 4 initial x86-64 FS-base control](step-4-initial-x86-fs-base.md)
- [Step 4 initial clear-child-TID word](step-4-initial-clear-child-tid.md)
- [Step 4 initial empty robust-list registration](step-4-initial-empty-robust-list.md)
- [Step 4 initial stack resource-limit queries](step-4-initial-stack-rlimit-query.md)
- [Step 4 initial thread signal mask](step-4-initial-signal-mask.md)
- [Step 4 bounded runtime anonymous page](step-4-bounded-runtime-anonymous-page.md)
- [Step 4 bounded runtime anonymous-page protection](step-4-bounded-runtime-anonymous-protection.md)
- [Step 4 bounded runtime anonymous-page advice](step-4-bounded-runtime-anonymous-advice.md)
- [Step 4 bounded monotonic `clock_gettime`](step-4-bounded-monotonic-clock-gettime.md)
- [Step 4 bounded monotonic `clock_getres`](step-4-bounded-monotonic-clock-getres.md)
- [Step 4 unsupported-syscall `ENOSYS` fallback](step-4-unsupported-syscall-enosys.md)
- [Step 4 bounded initial `sched_yield`](step-4-initial-sched-yield.md)
- [Step 4 bounded PCI configuration observation](step-4-bounded-pci-configuration-observation.md)
- [Step 4 bounded virtio-PCI capability discovery](step-4-bounded-virtio-pci-capability-discovery.md)
- [Step 4 bounded virtio-PCI feature negotiation](step-4-bounded-virtio-pci-feature-negotiation.md)
- [Step 4 bounded virtio-PCI queue discovery](step-4-bounded-virtio-pci-queue-discovery.md)
- [Step 4 bounded PCI bus-master authorization](step-4-bounded-pci-bus-master-authorization.md)
- [Step 4 bounded virtio DMA buffers](step-4-bounded-virtio-dma-buffers.md)
- [Step 4 bounded virtio split ring](step-4-bounded-virtio-split-ring.md)
- [Step 4 bounded DMA device preparation](step-4-bounded-dma-device-preparation.md)
- [Step 4 bounded virtio queue-address publication](step-4-bounded-virtio-queue-address-publication.md)
- [Step 4 bounded virtio queue activation](step-4-bounded-virtio-queue-activation.md)
- [Step 4 bounded virtio queue notification](step-4-bounded-virtio-queue-notification.md)
- [Step 4 bounded virtio completion signal](step-4-bounded-virtio-completion-signal.md)
- [Step 4 bounded virtio used-ring validation](step-4-bounded-virtio-used-ring-validation.md)
- [Step 4 bounded virtio payload completion](step-4-bounded-virtio-payload-completion.md)
- [Step 4 bounded entropy-sample capture](step-4-bounded-entropy-sample-capture.md)
- [Step 4 bounded entropy-sample quality attribution](step-4-bounded-entropy-sample-quality.md)
- [Step 4 bounded entropy-pool creation](step-4-bounded-entropy-pool-creation.md)
- [Step 4 bounded entropy-pool mixing](step-4-bounded-entropy-pool-mixing.md)
- [Step 4 bounded entropy-credit transfer](step-4-bounded-entropy-credit-transfer.md)
- [Step 4 bounded initial CRNG seeding](step-4-bounded-initial-crng-seeding.md)
- [Step 4 bounded CRNG extraction](step-4-bounded-crng-extraction.md)
- [Step 4 bounded initial `AT_RANDOM` delivery](step-4-bounded-initial-auxv-random.md)
- [Step 4 bounded initial `getrandom` delivery](step-4-bounded-initial-getrandom.md)
- [Step 4 reusable process `getrandom` service](step-4-reusable-process-getrandom.md)
- [Step 4 full-output process `getrandom`](step-4-full-output-process-getrandom.md)
- [Step 4 bounded small-request `getrandom`](step-4-bounded-small-request-getrandom.md)
- [Step 4 bounded large-request `getrandom`](step-4-bounded-large-request-getrandom.md)
- [Step 4 initial standard-descriptor close](step-4-initial-descriptor-close.md)
- [Step 4 initial standard-descriptor duplication](step-4-initial-descriptor-duplication.md)
- [Step 4 initial descriptor `fcntl` metadata](step-4-initial-descriptor-fcntl.md)
- [Step 4 initial descriptor `fstat` metadata](step-4-initial-descriptor-fstat.md)
- [Step 4 initial console `TCGETS`](step-4-initial-console-tcgets.md)
- [Step 4 initial console `TIOCGWINSZ`](step-4-initial-console-winsize.md)
- [Step 4 initial console `lseek`](step-4-initial-console-lseek.md)
- [Step 4 bounded multipage initial executable](step-4-bounded-multipage-initial-executable.md)
- [Step 4 bounded static ELF load plan](step-4-bounded-static-elf-load-plan.md)
- [Step 4 bounded static ELF page realization](step-4-bounded-static-elf-page-realization.md)
- [Step 4 bounded static ELF frame extent](step-4-bounded-static-elf-frame-extent.md)
- [Step 4 bounded static ELF mapping resources](step-4-bounded-static-elf-mapping-resources.md)
- [Step 4 bounded static ELF process entry](step-4-bounded-static-elf-process-entry.md)
- [Step 4 bounded static ELF initial `brk(0)` query](step-4-bounded-static-elf-initial-brk-query.md)
- [Step 4 bounded static ELF initial `brk` growth](step-4-bounded-static-elf-initial-brk-growth.md)
- [Step 4 bounded static ELF initial x86-64 FS base](step-4-bounded-static-elf-initial-x86-fs-base.md)
- [Step 4 bounded static ELF initial clear-child-TID word](step-4-bounded-static-elf-initial-clear-child-tid.md)
- [Step 4 bounded static ELF initial empty robust list](step-4-bounded-static-elf-initial-empty-robust-list.md)
- [Step 4 bounded static ELF initial `rseq` fallback](step-4-bounded-static-elf-initial-rseq-fallback.md)
- [Step 4 bounded static ELF initial stack-limit query](step-4-bounded-static-elf-initial-stack-limit-query.md)
- [Step 4 bounded static ELF initial `readlinkat` fallback](step-4-bounded-static-elf-initial-readlinkat-fallback.md)
- [Step 4 bounded static ELF initial `getrandom`](step-4-bounded-static-elf-initial-getrandom.md)
- [Step 4 bounded static ELF reusable `getrandom`](step-4-bounded-static-elf-reusable-getrandom.md)
- [Step 4 bounded static ELF post-random `brk(0)` query](step-4-bounded-static-elf-post-random-brk-query.md)
- [Step 4 bounded static ELF post-random `brk` growth](step-4-bounded-static-elf-post-random-brk-growth.md)
- [Step 4 bounded static ELF post-random `brk` page alignment](step-4-bounded-static-elf-post-random-brk-page-alignment.md)
- [Step 4 bounded static ELF RELRO `mprotect`](step-4-bounded-static-elf-relro-mprotect.md)
- [Step 4 bounded static ELF initial process-name query](step-4-bounded-static-elf-initial-process-name.md)
- [Step 4 bounded static ELF initial `getuid`](step-4-bounded-static-elf-initial-getuid.md)
- [Step 4 bounded static ELF initial configuration-stat miss](step-4-bounded-static-elf-initial-config-stat-miss.md)
- [Step 4 bounded static ELF startup credential normalization](step-4-bounded-static-elf-initial-credential-tail.md)
- [Step 4 bounded static ELF applet newline output](step-4-bounded-static-elf-applet-newline-write.md)
- [Step 4 pinned static BusyBox `echo`](step-4-pinned-static-busybox-echo.md)
- [Step 4 bounded static-process shell `getpid`](step-4-bounded-static-elf-shell-getpid.md)
- [Step 4 bounded static-process shell child disposition](step-4-bounded-static-elf-shell-sigchld-disposition.md)
- [Step 4 bounded static-process shell `getppid`](step-4-bounded-static-elf-shell-getppid.md)
- [Step 4 bounded static-process shell empty input](step-4-bounded-static-elf-shell-empty-input.md)
- [Step 4 bounded static-process shell exit command](step-4-bounded-static-elf-shell-exit-command.md)
- [Step 4 bounded static-process shell echo command](step-4-bounded-static-elf-shell-echo-command.md)
- [Step 4 bounded static-process shell console `fstat`](step-4-bounded-static-elf-shell-console-fstat.md)
- [Step 4 bounded static-process shell `/dev/tty` open miss](step-4-bounded-static-elf-shell-dev-tty-open-miss.md)
- [Step 4 bounded static-process shell standard-terminal probes](step-4-bounded-static-elf-shell-standard-terminal-probes.md)
- [Step 4 bounded static-process shell terminal warning](step-4-bounded-static-elf-shell-terminal-warning.md)
- [Step 4 bounded static-process shell `MAILPATH` metadata miss](step-4-bounded-static-elf-shell-mailpath-stat-miss.md)
- [Step 4 bounded static-process shell banner](step-4-bounded-static-elf-shell-banner.md)
- [Step 4 bounded static-process post-banner terminal probe](step-4-bounded-static-elf-shell-post-banner-terminal-probe.md)
- [Step 4 bounded static-process post-banner current directory](step-4-bounded-static-elf-shell-post-banner-getcwd.md)
- [Step 4 bounded static ELF leaf publication](step-4-bounded-static-elf-leaf-publication.md)
- [Step 4 bounded static ELF payload zeroing](step-4-bounded-static-elf-payload-zeroing.md)
- [Step 4 supplied static ELF artifact](step-4-supplied-static-elf-artifact.md)
- [Step 4 static ELF boot-resource transport](step-4-static-elf-boot-resource.md)
- [Step 4 bounded ELF identity and program-header facts](step-4-bounded-elf-identity.md)
- [Step 4 provider-neutral initial boot resource](step-4-initial-boot-resource.md)
- [Step 4 bounded external ELF load](step-4-bounded-external-elf-load.md)
- [Step 4 external ELF payload authority](step-4-external-elf-payload-authority.md)
- [Step 4 bounded external data and BSS](step-4-bounded-external-data-bss.md)
- [Step 4 runtime-derived executable extent](step-4-runtime-derived-executable-extent.md)
- [Step 4 runtime-derived data and BSS extent](step-4-runtime-derived-data-extent.md)
- [Step 4 runtime-derived ELF entry](step-4-runtime-derived-entry.md)
- [Step 4 published initial user image](step-4-published-initial-user-image.md)
- [Step 4 supplied initial user image](step-4-supplied-initial-user-image.md)
- [Generated inventory coverage](inventory/coverage.md)
- [Kernel design set](design/README.md)
- [Open decision register](design/decision-register.md)
- [Traceability](traceability.md)

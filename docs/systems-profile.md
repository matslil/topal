# Freestanding systems profile

The `systems` language feature extends Topal for kernels, hypervisors, firmware,
and similarly privileged freestanding artifacts. It preserves ordinary Topal
value, layout, effect, ownership, and safety semantics while adding a closed
set of machine-irreducible operations. It does not add an unsafe mode, raw
pointers, inline assembly, instruction templates, register allocation, or a
generic intrinsic mechanism.

The first implementation target is a Linux-compatible x86-64 kernel. AArch64
and RISC-V constrain the abstraction but are not implementation commitments.

## Selection and vocabulary

A source context selects `systems` through the existing language-feature field.
The feature adds the sealed `lang systems` namespace and no new grammar.
Systems declarations use ordinary Topal bindings, classifications,
applications, functions, layouts, effects, capabilities, and constructed
contexts.

Selection grants vocabulary, not privilege. A source package receives runtime
machine authority only when a qualified freestanding artifact provider
constructs its root context. Ordinary source cannot construct, copy, serialize,
deserialize, forge, or widen entry, CPU, address-space, interrupt, device, DMA,
fault-recovery, or artifact authority.

The feature is valid only for a freestanding root artifact. A package cannot
combine one systems context with the Linux-process artifact profile or acquire
host operating-system services implicitly. Dependencies which do not select
`systems` keep their portable meaning and receive no systems vocabulary or
authority merely because a systems root calls them.

Architecture models remain declarative and authority-free. They establish
whether a lowering is available and qualified; runtime capabilities identify
the actual processor, address space, interrupt domain, or device on which an
operation is authorized.

## Initial source vocabulary

The first executable-qualification slice uses ordinary Topal construction
syntax under the sealed `lang systems` namespace. It does not add keywords or
make the namespace extensible by source packages. The initial root shape is:

```topal
boot is fn (context : BootstrapContext) -> BootstrapDisposition
  context console write "Topal kernel booted"
  context debug break
  context fatal "bootstrap complete"

debug-break-handler is fn (
  context : DebugBreakContext
) -> DebugBreakDisposition
  context resume

lang systems artifact (
  bootstrap-storage is lang systems bounded-bootstrap-storage (
    capacity-bytes is 65536,
    alignment-bytes is 4096
  ),
  bootstrap is lang systems bootstrap-entry boot,
  debug-break is lang systems synchronous-exception-entry debug-break-handler
)
```

`BootstrapContext`, `BootstrapDisposition`, `DebugBreakContext`, and
`DebugBreakDisposition` are sealed systems classifiers, not constructible
ordinary values. `bootstrap-entry` specializes `SystemEntry Bootstrap`;
`synchronous-exception-entry` specializes `SystemEntry
SynchronousException` to the resumable debug-break cause used by the first
machine qualification.

`context console write` borrows the board-provided console session and accepts
a static text value in this first increment. `context debug break` is a
declared synchronous machine event whose selected provider transfers control
to the debug-break entry. `context resume` consumes that exception context and
resumes the interrupted bootstrap continuation. `context fatal` consumes the
bootstrap context and enters the nonreturning fatal provider. These members are
recognized only through their exact live context; they are not general
functions, namespace aliases, or values which can be stored or passed.

`bounded-bootstrap-storage` declares one artifact-provided monotonic pool. Its
capacity and maximum supported allocation alignment are positive byte counts;
the alignment is a power of two and cannot exceed the capacity. Allocation
through the live bootstrap context is fallible, never calls a host service, and
returns either one affine `BootstrapRegion` or the sealed storage error
`invalid-request`/`exhausted`. Released regions are consumed but their bytes are
not reused during bootstrap. The whole pool becomes reclaimable only after
bootstrap completes with no live regions. Requests carry semantic placement;
neither the pool nor a region exposes a physical address, object section, or
linker spelling.

The initial region-access form uses the language's ordinary exhaustive
`Result` and Boolean decisions:

```topal
context bootstrap allocate (
  byte-count is 64,
  alignment-bytes is 8,
  placement is bootstrap-reclaimable
)
  Ok region then {
    region byte store (offset-bytes is 0, value is 90)
    observed : Nat is region byte load (offset-bytes is 0)
    observed = 90
      true then {
        context bootstrap release region
        context fatal "complete"
      }
      false then {
        context bootstrap release region
        context fatal "memory mismatch"
      }
  }
  Error problem then context fatal "allocation failed"
```

`BootstrapRegion` is an affine capability for ordinary kernel-owned memory,
not a pointer or address. Byte load and store borrow the live region, require a
zero-based offset within its byte count, and produce or accept a `Nat` from 0
through 255. Release consumes the region after its last borrow. No operation
reveals the pool offset or permits the region to escape its bootstrap
execution. The initial executable slice admits only static requests and static
offsets whose alignment, capacity, value range, and bounds are proven before
lowering. A later dynamic offset must carry a proof or use a separately
approved explicit failure result; it never gains unchecked behavior.

These are plain-memory operations. They do not imply volatile, atomic, device,
DMA, firmware, or user-memory access, and cannot substitute for the separately
typed protocols governing those domains. Providers may choose different
concrete locations and addressing instructions while preserving the same
allocation, byte-content, lifetime, and failure observations.

### Boot-memory refinement

An entered bootstrap context owns an opaque provider-private boot handoff.
`context boot describe memory` consumes that context through an exhaustive
`Result` decision. Its success binding is a memory-described bootstrap context;
its error binding is a fatal-only failure context. The original context cannot
be used on either path after the transition.

The provider validates its native input and produces disjoint, nonempty,
nonwrapping physical ranges classified as allocatable, reclaimable,
persistent, reserved, unusable, or unknown. Source classifications and
provider provenance remain attached. Unknown input is never assumed to be RAM,
and overlapping claims are resolved conservatively before any physical-frame
capability is created.

Live image, adapter, stack, page-table, bootstrap-storage, handoff,
command-line, initramfs, and firmware ranges are reserved automatically.
Page-edge fragments which do not form complete frames also remain reserved.
Portable source sees range capabilities rather than firmware records or
numeric addresses; later frame allocators consume those capabilities.

The portable meaning applies equally to an x86 E820 handoff, AArch64 Device
Tree or UEFI descriptors, and RISC-V Device Tree or SBI/platform facts. Each
provider owns parsing and target evidence. Malformed, cyclic, overflowing, or
unsupported input fails closed without exposing the provider representation.

The first target identity is `x86_64-unknown-none`, board
`topal-qemu-pc-q35-10.2`, under profile
`topal.systems.x86_64-qemu-pc-q35-10.2/1`. It is executable-qualified only for
its closed generated entry, provider, storage, linked-artifact, and Linux boot-
adapter set under the pinned QEMU evidence. Unsupported operations, targets,
boards, CPUs, models, or publication options continue to fail before output.

### Physical-frame allocation

`context memory create frame allocator` consumes a memory-described bootstrap
context through an exhaustive `Result` decision. Success produces one
`FrameAllocatorContext` which retains the bootstrap capabilities and owns the
normalized allocatable-frame authority. Failure produces only a fatal-capable
allocator-failure context. The consumed memory-described context cannot be
reused on either path.

An allocation requests a nonzero frame count and power-of-two alignment in the
target profile's base-frame units. Success returns an affine opaque physical-
frame extent. The extent records its allocator identity, count, alignment, and
provider provenance without exposing a physical base. It grants ownership, not
memory access or mapping authority. Exhaustion or a malformed request is an
explicit failure and leaves the allocator usable.

Release consumes an extent and returns it only to the allocator which created
it. Live extents do not overlap each other or boot reservations. They cannot
escape the bootstrap entry, cross allocator identities, be released twice, or
remain live at a disposition. Allocation strategy and coalescing are ordinary
library policy so long as these observations are preserved.

X86-64 base pages, AArch64 translation granules, and RISC-V page/Sv modes are
provider facts. Portable source counts the profile-selected complete frames;
it does not name page-table formats, numeric byte addresses, or target page
sizes. The first executable x86-64 slice admits one statically proved frame
aligned to one frame. Multiple live extents, dynamic requests, and reclamation
remain unavailable. Mapping is admitted only through the separately qualified
opaque capability below.

### Opaque kernel mapping

`context kernel map frames (...)` borrows the live allocator context and
consumes one affine physical-frame extent. The initial request names semantic
rights, execution permission, and memory kind:

```topal
memory kernel map frames (
  rights is read-write,
  execution is denied,
  memory-kind is normal
)
  Ok mapping then {
    mapping byte store (offset-bytes is 0, value is 165)
    observed : Nat is mapping byte load (offset-bytes is 0)
    frames is memory kernel unmap mapping
    memory frames release frames
    memory fatal "complete"
  }
  Error problem then {
    memory fatal "kernel mapping failed"
  }
```

Success produces one affine `KernelMapping` which owns the frame extent while
live and records an opaque provider-selected kernel-virtual extent, rights,
memory kind, owner, lifetime, and translation-provider evidence. It reveals
neither range as a number. The initial mapping grants bounded plain byte load
and store only for read-write, non-executable normal memory. It does not grant
physical, user, device, DMA, firmware, volatile, atomic, or executable access.

`kernel unmap` consumes the mapping and returns exactly its original opaque
frame extent after the provider proves that the mapping no longer authorizes
access. The extent can then be released to its allocator. Frame release while
mapped, mapping use after unmap, cross-provider use, writable execution,
out-of-bounds access, and a disposition with a live mapping are invalid.

The first x86-64 provider may adopt its sealed bootstrap identity mapping for
one selected frame below the qualified identity-map limit. This is a provider
implementation fact, not portable identity between physical and virtual
families. AArch64 and RISC-V providers may adopt or construct different
translations while preserving the same ownership, rights, access, unmap, and
reuse observations. Page-table builders, activation, permission changes,
translation invalidation, multiple mappings, and user mappings remain
separate fail-closed increments.

## Special entries

A `SystemEntry K` is a static artifact declaration for one closed entry kind
`K`, not an ordinary callable function. Initial kinds cover bootstrap CPU,
secondary CPU, synchronous exception, external interrupt, syscall from user
mode, non-maskable or machine-critical event, and resumed kernel thread.

Each entry binds:

- one qualified target/profile and artifact-visible identity;
- an opaque context type and capabilities legal in that context;
- a handler with explicit effects and resource requirements;
- legal nesting, masking, preemption, allocation, blocking, and failure rules;
  and
- a closed set of dispositions such as resume, return to user, schedule,
  deliver a user exception, or enter the fatal path.

The compiler/backend owns the physical symbol, section, alignment, machine
frame, stack transition, saved state, security sequence, unwind information,
and return mechanism. Context accessors expose semantic facts only when the
entry kind defines them. An entry context is affine, cannot escape its handler,
and cannot cross a task/message, storage, foreign, or serialization boundary.

A disposition is accepted only after the checker proves that acknowledgement,
resource, recovery, masking, preemption, and context-validation obligations are
discharged. Source does not select a machine return instruction.

## External observations and permitted choice

Systems code may observe nondeterminism only through a declared observation
source or concurrent protocol. Interrupt arrival, device completion, clock
progress, user input, another CPU's ordered operation, and scheduler-visible
wake events each name an exact source capability, value or error layout,
ordering constraints, and trace identity.

When concurrently enabled transitions have no required order, the owning
protocol may observe any permitted linearization. Every transition preserves
the protocol invariant and every result must satisfy the declared external
contract. Replay evidence records the observations and selected linearization.
The compiler may neither invent nor discard an observation.

This is the systems-profile interpretation of permitted nondeterminism under
`TOPAL-REQ-DETERMINISM-001`. It does not make data races meaningful or add a
general random choice. Portable functions retain deterministic meaning unless
one of their explicit inputs or effects carries a declared observation.

## Machine providers

Privileged operations are closed functions in qualified machine-provider
interfaces. Every operation states its consumed and produced semantic state,
required authority and resource identity, legal execution context, fault
behavior, ordering relation, target evidence, and abstract model transition.

Common operation families express semantic intent: install a validated entry
table, activate an address space, invalidate translations in a stated scope,
route an interrupt, wait under a declared wake contract, or publish modified
instructions. A common name exists only when architecture providers implement
the same contract.

Facilities without a shared meaning remain target-qualified, including x86
port I/O and model-specific state, architecture-specific system registers, and
virtualization controls. Target-specific operations remain typed and
capability-gated; “target-specific” never means unchecked.

## Address spaces and mappings

Systems address families are opaque and resource-qualified. Physical, kernel
virtual, user virtual, device/MMIO, DMA/IOMMU, and firmware-source addresses
remain distinct even when their numeric representations coincide.

An address range carries a checked target width and nonwrapping bounds. A
location combines one range with a layout, rights, lifetime, supported access
sizes/alignment, cache policy, and ordering domain. A mapping is a linear
resource relating address ranges under a page layout, permissions, owner,
lifetime, and translation provider.

Usable addresses arise only from validated boot or firmware input,
allocation, mapping, bus resources, or checked derivation from a live range.
Decoding an external number produces an untrusted candidate, not access
authority. There is no general integer-to-address or address-family cast.

Page-table construction uses an exclusive builder. Activation, protection
change, unmapping, and reuse require the mapping and target-qualified
translation/invalidation evidence appropriate to every affected processor.

### Translation-space construction and activation

The initial construction vocabulary exposes the builder lifecycle without
exposing a page-table format:

```topal
memory translation begin (
  template is bootstrap-equivalent,
  page-policy is provider-selected
)
  Ok update then {
    memory translation commit update
      Ok space then {
        memory translation activate space
          Ok translated then {
            translated console write "TOPAL_KERNEL_TRANSLATION_ACTIVE"
            translated fatal "complete"
          }
          Error failure then {
            failure fatal "translation activation failed"
          }
      }
      Error failure then {
        failure fatal "translation commit failed"
      }
  }
  Error failure then {
    failure fatal "translation construction failed"
  }
```

`translation begin` borrows the live frame allocator, reserves the
provider-selected backing required by the qualified target, and returns one
affine exclusive `TranslationUpdate`. The update owns that backing and a
snapshot of the requested semantic mapping template. It exposes neither table
entries nor backing addresses. The initial template is exactly
`bootstrap-equivalent`: it preserves the current bootstrap coverage and access
observations. It does not add a user mapping, widen a permission, or imply that
physical and virtual identities are interchangeable.

`translation commit` consumes the update after provider validation and returns
one inactive affine `TranslationSpace`. Commit failure consumes the incomplete
builder into a fatal-only failure context in the initial slice. `translation
activate` consumes the inactive space and the current translation authority,
performs the target-qualified publication, synchronization, activation, and
completion protocol, and returns a refined bootstrap context owning the new
active space. The previous bootstrap translation remains provider-owned and
unavailable to ordinary source.

Builder duplication or escape, activation before commit, use after commit,
activation of the wrong provider space, backing-frame release while owned by
an update or space, and a recoverable continuation after an indeterminate
activation all fail closed. The first x86-64 provider constructs a replacement
four-level root using provider-owned frames and 2 MiB identity leaves for the
already qualified first-GiB bootstrap coverage, then activates it with the
required control-state transition. AArch64 and RISC-V providers may select
different granules, levels, descriptor formats, and maintenance sequences
while preserving the same builder, commit, activation, coverage, permission,
and ownership observations.

This initial operation replaces a bootstrap-equivalent translation only. It
does not yet create arbitrary ranges, change permissions, construct user
spaces, support multiple active spaces, or expose general invalidation.

### Active translation edits

Mappings on an active translation space change only through an exclusive
transaction. The initial map lifecycle is:

```topal
active translation edit begin
  Ok edit then {
    edit kernel map frames (
      rights is read-write,
      execution is denied,
      memory-kind is normal,
      placement is provider-selected
    )
      Ok mapping then {
        edit translation commit
          Ok edited then {
            mapping byte store (offset-bytes is 0, value is 60)
            edited fatal "complete"
          }
          Error failure then {
            failure fatal "translation edit commit failed"
          }
      }
      Error failure then {
        failure fatal "translation edit mapping failed"
      }
  }
  Error failure then {
    failure fatal "translation edit construction failed"
  }
```

`translation edit begin` consumes the active context into one affine
`TranslationEdit`; ordinary operations through the old context are unavailable
until commit succeeds. Mapping consumes a live physical-frame extent into one
provisional opaque `KernelMapping`. The mapping grants no access before
`translation commit` consumes the edit, publishes the target-qualified
translation changes, completes required ordering and invalidation, and returns
a refined active context. A failed or indeterminate commit is fatal-only.

Unmapping uses a second exclusive edit. It consumes the live mapping into a
provisional returned frame extent. That extent cannot be accessed, released,
or remapped until commit removes the translation, performs the complete
target-qualified invalidation protocol, and returns the next refined context.
The mapping becomes unusable when unmap enters the transaction, not after a
later best-effort cleanup.

The initial placement is exactly `provider-selected`: source observes only the
mapping capability, bounds, rights, memory kind, and ownership transitions. It
cannot observe or choose a virtual address, page size, table level, entry,
address-space identifier, invalidation address, processor mask, register, or
instruction. The first x86-64 provider admits one 4 KiB normal read-write,
non-executable mapping in a reserved kernel window and invalidates it before
returning its frame. AArch64 and RISC-V providers may use different granules,
levels, address-space identifiers, barriers, and invalidation scopes while
preserving the same transaction, visibility, and ownership observations.

This slice does not yet admit concurrent edits, multiple live dynamic
mappings, user mappings, device memory, permission changes, executable
mappings, remote-processor shootdown, or reclamation of provider metadata.

## Fault-contained access

User addresses are untrusted ABI values rather than Topal references. A user
transfer names the current user-address-space capability, candidate range,
maximum extent, direction, selected layout or byte policy, kernel-owned
storage, and partial-progress/interruption policy.

The compiler and provider establish the target access state and generated
recovery sites. The operation returns validated data, permitted partial
progress, or a structured fault. Source cannot name a recovery instruction
address or install a catch-all exception handler.

A recovery scope admits only its declared synchronous faults and operations.
Unexpected kernel faults, machine-critical events, stack corruption, and
faults outside the generated scope use their owning fatal or entry protocol.

## Atomic locations and shared state

`AtomicLocation T D` is a live, suitably aligned fixed-layout location for one
target-supported scalar or tagged state `T` in synchronization domain `D`.
It is created from exclusively owned storage and must end before that storage
is released or reused non-atomically.

The closed operations are atomic load, store, exchange, compare/exchange, and
qualified read-modify-write. They support these semantic orders:

- `AtomicOnly`: indivisibility and one per-location modification order;
- `Acquire`: later observations follow the release sequence observed;
- `Release`: prior observations precede a successful observer acquire;
- `AcquireRelease`: both relations for a modifying operation; and
- `Sequential`: acquire/release plus one global order among sequential
  operations.

Compare/exchange declares success and failure order separately, and failure
cannot release. Operations also state the compiler, CPU, device, and DMA
domains involved; a CPU atomic does not imply MMIO or DMA ordering.

Atomic winner selection is permitted only inside a declared protocol whose
invariant holds for every transition. Non-atomic conflicting access remains a
rejected race. Mixed access requires a proved ownership transition which ends
all atomic access first.

The initial source form specializes that lifecycle to one unsigned machine
word in CPU-shared normal memory:

```topal
atomic is region atomic word create (
  offset-bytes is 8,
  initial-value is 41,
  domain is cpu-shared
)
atomic compare exchange (
  expected is 41,
  desired is 42,
  success-order is acquire-release,
  failure-order is acquire
)
  Exchanged previous then {
    observed : Nat is atomic load (order is acquire)
    observed = 42
      true then {
        region is atomic end
        restored bootstrap release region
      }
      false then {
        region is atomic end
        restored bootstrap release region
        restored fatal "atomic load mismatch"
      }
  }
  Observed actual then {
    region is atomic end
    restored bootstrap release region
    restored fatal "atomic compare exchange lost"
  }
```

`atomic word create` consumes the entire region even though only its selected
word is the atomic location, so no plain alias remains usable. `atomic end`
consumes the location, proves that no operation remains in flight, retains the
final word contents, and returns that same region to plain ownership. The
machine-word width is supplied by the qualified target; source cannot select
an instruction width or address. The initial `cpu-shared` domain excludes
MMIO, device, DMA, firmware, and user memory.

Mutex, spin-style, sequence, reference-count, epoch/RCU, wait, and completion
algorithms are libraries over atomic and critical-scope elements. The language
does not standardize one universal lock.

## Critical scopes and visibility

Interrupt masking and scheduler-preemption control are scoped, affine state
transitions tied to one processor and domain. Entering returns a restoration
token containing the exact previous state. Every exit path consumes it once;
it cannot cross processors, suspend, escape, or restore out of nesting order.

The initial local-interrupt form consumes the current execution context and
returns a context refined to `local-maskable-interrupts`:

```topal
context critical enter (
  domain is local-maskable-interrupts
)
  Ok critical then {
    critical console write "TOPAL_KERNEL_INTERRUPTS_MASKED"
    restored is critical restore
    restored console write "continued"
  }
  Error failure then {
    failure fatal "critical entry failed"
  }
```

`critical restore` consumes the refined context and returns the context state
that existed immediately before the matching entry. Nested scopes, when a
provider admits them, restore in last-in-first-out order. Neither source nor
the common semantic model can observe the saved flags, registers, masks, or
instructions. A local-interrupt critical context cannot be used for an
operation which may suspend or block, and it must be restored before control
can escape its lexical continuation.

Masking local interrupts does not exclude another processor, an NMI, DMA, or a
device. An exclusion proof names the exact producers removed by the scope.
Potentially blocking acquisition is invalid in interrupt and machine-critical
contexts.

Visibility operations separately describe CPU memory order, MMIO/device order,
DMA ownership, cache maintenance, instruction synchronization, and translation
maintenance. Each names the observations, direction, agents, topology scope,
and completion point. A target may lower a relation to zero instructions only
when qualified evidence establishes the complete contract.

## Scheduler context transfer

A suspended context is an opaque linear resource owning one kernel stack,
saved machine state, extended-state policy, thread identity, and address-space
relationship. It arises only from entry, initial-thread construction, or a
prior transfer.

Context transfer consumes the running-context capability and one validated
suspended context. It applies the scheduler's per-CPU and address-space
transition and resumes exactly one continuation. It is nonordinary control
flow: it does not return as a normal function, although a later transfer may
resume the old typed continuation point.

Source cannot inspect or construct register slots. Debugger and user-process
ABI adapters use separate validated semantic state views.

## Devices and DMA

Device locations extend ordinary layouts with one register protocol containing
legal widths, access direction, read/write side effects, reserved-bit policy,
ordering, and device-session lifetime. Device access is an observable effect
and cannot be duplicated, removed, invented, or moved across a conflicting
protocol event.

A DMA buffer moves linearly through CPU-owned, prepared, device-owned,
completed or failed, and CPU-owned states. Each transition records direction,
device and translation domain, mapping, cache maintenance, descriptor
visibility, notification, completion source, and reclamation. Coherent and
noncoherent providers implement the same ownership contract; a no-operation
maintenance path still needs qualified evidence.

## Storage, placement, and artifacts

The systems root begins with bounded bootstrap storage and may later construct
allocator capabilities. Allocation is fallible and names a region or pool,
alignment, address family, context legality, reclaim policy, and any physical
or DMA constraints. No allocation or failure path calls a host operating
system implicitly.

Ordinary kernel-owned regions support bounds-checked plain byte load and store
while borrowed and become inaccessible when consumed by release. They expose
neither their address nor their provider's allocation metadata. Volatile,
atomic, device, DMA, firmware, and user-memory operations remain distinct typed
protocols rather than flags on a plain region access.

Static placement is semantic evidence such as special-entry text, read-only
data, mutable data, per-CPU template, bootstrap-reclaimable data, page-aligned
table, or boot-adapter requirement. Source does not spell object-section names
or linker directives. The artifact provider chooses a representation and
proves the placement constraint.

A systems artifact uses a distinct target profile with no host syscall, libc,
process startup, dynamic loader, or implicit foreign runtime. It publishes
linked kernel, debug, map, and provenance outputs atomically after structural
validation. A qualified packaging adapter then creates the selected
firmware/bootloader image; the language does not assume one boot protocol.

An unrecoverable disposition enters a target-qualified nonreturning fatal
provider. The minimal fatal path requires no general allocation, blocking lock,
scheduler, filesystem, or userspace service.

## Qualification

Each systems element requires:

1. architecture-independent semantic and negative tests;
2. rejection for wrong feature, artifact, context, authority, lifetime, order,
   target, or unsupported provider;
3. an abstract interpreter/model transition or an explicit model-only
   diagnostic;
4. backend artifact inspection for entry, placement, relocation, undefined
   dependencies, unwind/debug state, and privileged instruction confinement;
5. target-provider tests in an emulator or physical qualification environment;
   and
6. exact model, provider, toolchain, board, and assumption provenance.

The first executable qualification is x86-64 only. Architecture-neutral
semantics continue to be reviewed against AArch64 and RISC-V so x86 mechanisms
do not become universal source meaning.

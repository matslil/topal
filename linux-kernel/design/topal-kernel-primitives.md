# Criteria for kernel-capable Topal elements

This document is a design input, not approved Topal language meaning. Concrete
syntax and semantics require prior discussion and authoritative updates under
the repository change procedure.

## TK-ELEMENT-001 — Abstraction rule

New Topal elements shall expose the smallest semantic operation needed to state
and verify kernel behavior. They shall not expose source-level assembly,
instruction templates, clobber lists, calling-convention register allocation,
or opaque backend intrinsics.

Portability is based on shared meaning, not shared spelling:

- use one portable element when architectures provide the same observable
  operation under a common contract;
- use a common interface with target-qualified implementations when preconditions
  or strengths differ but meaning can be stated uniformly; and
- use an architecture-specific element when no honest portable contract exists.

The backend may use machine instructions, object-format directives, and LLVM
facilities internally. Those choices are implementation evidence and cannot be
selected or observed as ordinary source values.

## Required semantic dimensions

Every proposed element identifies:

- privilege and unforgeable authority;
- processor, core, address-space, device, or other resource identity;
- input/output value and representation constraints;
- legal entry state and resulting state transition;
- failure, fault, interruption, retry, and cancellation behavior;
- effect identity, ordering, atomicity, visibility, and synchronization scope;
- preemption, interrupt-context, blocking, and reentrancy rules;
- lifetime and ownership of referenced storage or capabilities;
- target-model evidence needed for lowering;
- interpreter/model behavior where meaningful; and
- code-generation, negative, differential, and hardware/VM tests.

## Candidate element families

These families identify design work; their names are descriptive placeholders.

### Entry and execution contexts

Typed boot entry, secondary-processor entry, trap/interrupt entry, syscall
entry, context switch, and return transitions. The compiler owns the exact
machine frame and special prologue/epilogue. Source handlers receive a validated
context value and only the capabilities legal in that context.

### Address spaces and translation

Distinct physical, kernel-virtual, user-virtual, device, DMA, and firmware
addresses; mappings with permissions, cache/access policy, lifetime, and owner;
page-table construction; activation; faults; translation invalidation; and safe
user transfer. Numeric equality alone must not make addresses interchangeable.

### Privileged processor state

Typed architecture state identities and checked read/write/transition
operations. Portable operations should express intent such as installing an
exception table, activating a translation space, changing an interrupt-mask
scope, invalidating translations, or waiting for an event. CPUID leaves, x86
MSRs, AArch64 system registers, and RISC-V CSRs remain target-qualified where
their meaning is not portable.

### Interrupts, exceptions, and external observations

Interrupt sources, routing, priority, acknowledgement, masking, completion,
nesting, affinity, and handler contexts; synchronous faults with typed recovery
or termination; and external device/timer observations whose nondeterminism is
declared without introducing undefined behavior.

### Shared kernel synchronization

The design must decide which synchronization remains compiler-synthesized and
which operations become explicit kernel elements. Required use cases include
interrupt-safe exclusion, per-CPU data, wait/wake, RMW state, reference and
epoch management, lock-free device queues, futex implementation, and ordering
with DMA or MMIO. C-style atomics are not assumed to be the right source model.

### Device and transfer operations

Validated MMIO and x86 port-I/O locations, access sizes, register semantics,
barriers, DMA ownership, scatter/gather, IOMMU mappings, cache maintenance,
doorbells, and completion. Operations preserve device protocol and effect
identity instead of acting as generic volatile pointers.

### Kernel storage and failure

Bootstrap/static storage, page and object allocators, bounded stacks, scoped
regions, fallible allocation, non-returning fatal termination, and recovery
boundaries. No facility may depend on a host Linux syscall or implicit foreign
runtime.

## Cross-architecture pressure test

The semantic design shall be reviewed against at least x86-64, AArch64, and
RISC-V before approval, even though only x86-64 is implemented.

| Concern | x86-64 | AArch64 | RISC-V | Abstraction consequence |
| --- | --- | --- | --- | --- |
| privilege | rings and system state | exception levels | machine/supervisor/user modes | privilege is a capability/state, not a ring number |
| trap vector | IDT gates and stack rules | vector base and exception classes | trap vector and cause CSRs | typed event entry plus target-qualified frame |
| translation root | control-register state | translation table base/control registers | `satp` and mode | activate a checked address space |
| invalidation | page/context invalidation operations | scoped TLBI operations | `SFENCE.VMA` scopes | semantic range/address-space invalidation |
| interrupt masking | flags and controller state | DAIF/controller state | status bits/controller state | scoped mask capability with explicit domain |
| low-power wait | halt/monitor families | WFI/WFE families | WFI | wait under declared wake/progress contract |
| addressed I/O | MMIO and port I/O | primarily MMIO | primarily MMIO | port I/O remains target-specific |
| virtualization | VMX/SVM | EL2 facilities | hypervisor extension | later target-qualified virtualization interface |

The comparison prevents x86 register names or instruction quirks from becoming
the universal source model. It also prevents the opposite error: inventing a
weak generic operation that cannot state the guarantees required by any real
architecture.

## First design deliverable

Before kernel code, each candidate family needs:

1. examples of required Linux behavior;
2. comparison across the three architecture families;
3. proposed Topal semantic objects and operations;
4. interaction with existing effects, resources, layouts, tasks, contracts,
   and architecture evidence;
5. rejected alternatives and compatibility consequences; and
6. an approval checkpoint before changing `docs/` or `se/`.

Step 2 supplies that proposal in the [systems-profile element set](kernel-elements.md),
[cross-architecture pressure test](architecture-pressure-test.md),
[kernel architecture](topal-kernel-architecture.md), and
[Linux mapping ledger](linux-to-topal-mapping.md). The records were approved
after PR #790 and are propagated through the authoritative systems-profile
design, requirements, and formal specification.

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
requirements, and formal rules. This does not claim that the present compiler
can build a kernel: implementation and executable x86-64 qualification remain
planned, and the current compiler remains a Linux x86-64 userspace compiler.

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
  labs/qemu/x86_64/           future reproducible reference/test VM
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
- [Generated inventory coverage](inventory/coverage.md)
- [Kernel design set](design/README.md)
- [Open decision register](design/decision-register.md)
- [Traceability](traceability.md)

# Bounded static ELF address-space activation and restoration

This record resolves `TK-DEC-137` as the first active-root transition for the
approved static ELF address space. It is a systems-profile refinement only:
checked source owns opaque address-space lifecycle states, while providers own
root registers, translation barriers, invalidation, and target page-table
representation.

## Decision

`static image construction activate address space` shall consume the exact
live `AttachedStaticImageConstruction` from `TK-DEC-136` and produce one affine
`ActiveStaticImageConstruction`. It shall accept only activation policy
`bounded-static-elf-kernel-continuation`. Before switching roots, the provider
shall revalidate the complete attached hierarchy, all final leaves, both root
identities, and that the static root is inactive while the qualified bootstrap
root is active.

Activation shall atomically select the static root with target-required
translation synchronization, then read back that it is active. Source shall
continue only through the already mapped supervisor kernel path and emit
exactly one `TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK` witness. This
transition grants no user entry, stack, TLS, signal, or process authority and
shall not execute a static-image byte.

`static image construction restore address space` shall consume only that
exact active owner. It shall accept only restoration policy
`bounded-bootstrap-kernel-continuation`, prove the static root active and the
qualified bootstrap root distinct, atomically restore the bootstrap root with
target-required synchronization, and read back both that the bootstrap root is
active and the static root is inactive. Success shall reproduce one affine
`AttachedStaticImageConstruction` owning the unchanged payload, leaves, and
hierarchy. Source shall emit exactly one
`TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK` witness before the existing
detach, zero, reclaim, and release sequence.

Any identity, hierarchy, leaf, active-root, readback, or synchronization
failure is fatal. No active owner may reach a recoverable disposition, detach,
reclaim, or release. The bootstrap root remains provider-verified restoration
authority; source never observes a numeric root.

## Exact x86-64 realization

The checked root retains the qualified bootstrap translation-root identity and
passes it with the attached static hierarchy identity. The provider shall:

- revalidate the exact hierarchy and 503 final leaves established by
  `TOPAL-SYSTEMS-STATIC-ELF-ADDRESS-SPACE-001`;
- require current `CR3` to name the bootstrap root before activation;
- load the static hierarchy root into `CR3`, then read back the exact root;
- on restoration, require current `CR3` to name the static root;
- load the retained bootstrap root into `CR3`, then read back the exact root;
  and
- revalidate that the static root is inactive without modifying any hierarchy,
  leaf, or payload byte.

The static hierarchy's existing supervisor identity mappings keep the kernel
instruction stream, stack, provider text, and retained affine state reachable
during the bounded active interval. Qualified HPET and APIC mappings remain
available. The static user leaves remain unchanged and no transition derives,
publishes, or branches to their entry point.

## Affine lifecycle and rollback

Activation consumes the sole attached authority; restoration consumes the sole
active authority and returns the attached state. Duplicate, substituted,
stale, nested, out-of-order, or live-at-disposition authority is invalid. Only
after successful restoration may the existing detach operation clear the
hierarchy and return its resources. Failure after a root switch cannot return a
partial owner and enters the fatal halt in the currently active, fully
qualified mapping.

## Architecture pressure test

Opaque activation/restoration authority, exact active-root readback, a bounded
kernel-only continuation, and explicit restoration before teardown are portable
policy. AArch64 providers may update `TTBR` registers and execute required
`DSB`/`ISB` sequences. RISC-V providers may update `satp` and execute the
required `SFENCE.VMA`. Capability systems may enter and leave a sealed
address-space object rather than write an integer register. Source names none
of those mechanisms.

## Alternatives rejected

Entering the static user image during activation was rejected because stack,
TLS, entry, syscall, and process authority are not yet established. Keeping the
static root active through later unrelated qualification was rejected because
it would widen rollback and obscure which mappings support the existing probe.
Letting source store the prior numeric root was rejected because roots are
provider-private capabilities. Restoring without readback was rejected because
teardown must not clear an active hierarchy. Treating activation as an ordinary
console-witness side effect was rejected because root ownership must remain
affine and explicit.

## Deliberate bounds

This increment activates one already qualified root, executes only the bounded
kernel continuation and witness, restores the prior root, and then uses the
existing detach path. It does not retain the static root as a process address
space, allocate a stack, initialize TLS, publish the static entry, enter user
mode, or run BusyBox.

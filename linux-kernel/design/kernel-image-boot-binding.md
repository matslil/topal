# Kernel-image boot binding

`TK-DEC-289` is a systems-only prerequisite for protected integration under
the standing authorization. It strengthens inactive table preparation without
changing source syntax, core semantics or the backing-only/legacy profiles.

The ELF image does not contain the boot adapter's initial translation pages,
GDT, IDT, descriptor pointer or stack. Those retained roles must be bound to the
actual source-built adapter, with real handler targets inside executable image
roles. The transition code ends in a jump to kernel entry, not a call; its code
role ends at that handoff, while its bytes and initial tables remain retained.
This is not permission to reclaim boot-reserved state.

The private producer reconstructs the whole adapter from the fresh kernel and
compares its bytes before loading. Native preparation validates the actual entry
root and every entry in its four paging pages, the generated transition bytes,
GDT/IDT and their live registers, exact handler targets, stack and allocator's
retained handoff/resource facts. Hardware accessed/dirty bits are masked only
where applicable. No control register or descriptor register is written.

Capabilities gate optional MSR reads. The bounded x86 profile records CR0/CR3/
CR4, EFER, APIC identity, PAT, MTRR capability/default state, every advertised
variable range within its supported bound, and fixed ranges when advertised.
The same state and identities are checked again before discard. The initial
profile admits PAE/LA48 without PGE/PCIDE or other CR4 extensions; it observes
WP/NXE without claiming they are already effective protected permissions.

The primary direct-window and APIC/HPET cache indices are preserved from the
checked entry topology. Splitting a large image leaf retains its applicable
PAT/PWT/PCD index. The boot-role record distinguishes the direct window,
retained tables/descriptors/stack, closed handoff code and MMIO views; it does
not turn every address into an owned ordinary RAM region or qualify hardware
attributes from a section name.

All physical CPUs are observed independently in the VM without state injection.
Those snapshots are evidence, not an AP parking or all-CPU lifetime witness.
All usable roots, writers, effective controls, invalidation, active/restorable
context eligibility, asynchronous activity and fatal/retirement obligations
still precede activation. No topology restriction is adopted to bypass them.

Primary architectural reference: [Intel SDM Volume 4, architectural MSRs](https://www.intel.com/content/dam/develop/external/us/en/documents/335592-sdm-vol-4.pdf),
including capability-gated MTRR/PAT enumeration. The current Volume 3A download
timed out during this stage; register definitions are verified against the
available manufacturer reference rather than inferred from that failed fetch.

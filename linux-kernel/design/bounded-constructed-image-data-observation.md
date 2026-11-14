# Bounded constructed-image data observation

This record resolves `TK-DEC-049` as a systems-profile refinement. It requires
the provider-sealed entry payload in the finalized code-role mapping to observe
the exact finalized data-role mapping before transferring to the existing
qualified Linux syscall and fault probe. It changes no core-language rule and
does not expose a general code/data relocation or addressing interface.

## Decision

The existing entry-trampoline installation and verification operations retain
their architecture-neutral meaning: they install and verify the exact opaque
payload selected by the provider. For this bounded profile, that payload must:

1. begin execution in the finalized code-role mapping;
2. perform one read-only observation of the provider-sealed sentinel in the
   finalized immutable-data mapping;
3. fail closed without entering the qualification probe if the observation is
   wrong; and
4. transfer to the unchanged provider-owned qualification probe only after the
   observation succeeds.

After the exact kernel return, source publishes
`TOPAL_KERNEL_USER_IMAGE_ENTRY_OK`, then
`TOPAL_KERNEL_USER_IMAGE_DATA_OK`, and only then the existing residency
witness. Both role owners remain live through all three observations and are
subsequently revoked data first and code second.

```text
finalize code payload RO/X + sentinel data RO/NX
  -> image-ready witness -> enter at code mapping
  -> read sealed data sentinel -> transfer to qualified probe
  -> exact kernel return -> entry witness -> data witness
  -> residency witness -> data-first teardown
```

Source names neither the sentinel value nor either virtual address. The
provider binds payload encoding, placement, observation encoding, failure
instruction, transfer destination, and any cache or instruction-stream
synchronization. Structural evidence must prove the installed and verified
payloads agree exactly and that the data access is a read, not a write.

## Isolation and failure contract

The data mapping remains RO/NX throughout execution. The code payload may read
only the sealed sentinel needed for this bounded observation; it may not write
data, execute data, derive a source-visible address, or retain a pointer. A
wrong sentinel, wrong mapping, skipped comparison, transfer before comparison,
early or missing data witness, mutation after verification, or teardown before
the witness is invalid. Fatal-only completion remains in force after
permission publication begins.

## Cross-architecture pressure

The portable contract is a read-only observation from one finalized image role
by another, followed by fail-closed control transfer. It does not prescribe an
instruction, register, immediate, address, branch range, or relocation.

An x86-64 provider may materialize the private data placement and compare a
byte before an indirect transfer. AArch64 may use PC-relative or literal-based
address materialization and must preserve its cache/barrier publication
contract. RISC-V may use a PC-relative sequence and the required
instruction-stream synchronization. Capability architectures would need a
provider-derived read-only capability rather than an integer address. In all
cases source observes only success after exact kernel return.

This increment does not admit arbitrary constants, offsets, data layouts,
mutable data, relocations, ELF loading, arbitrary segments or aliases, demand
paging, mapping ownership transfer, or a general process model.

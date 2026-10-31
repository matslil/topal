# Provider-neutral initial boot-resource contract

This record resolves `TK-DEC-059` as a systems-feature refinement. It admits
one immutable boot resource as provider-private input to the existing
boot-memory and initial user-address-space transitions. It changes no
core-language rule and exposes neither a firmware address nor an archive
format to portable Topal source.

## Decision

The initial kernel profile may receive one bounded immutable boot resource
from its board adapter. Boot-memory refinement must validate the native
handoff, retain the resource as a reservation, exclude its complete physical
extent from allocation, and carry its authority privately with the refined
bootstrap context. An absent, empty, overflowing, mutable, multiply described,
or out-of-range resource fails closed.

The sealed `initial-probe` user-address-space layout may require that retained
resource. Its provider must validate the complete resource identity before it
constructs or enters user mappings. Portable source selects the semantic
layout only; it cannot inspect the resource bytes, recover its address, choose
an archive parser, or weaken the validation.

```text
board adapter supplies immutable resource
  -> boot-memory refinement validates and reserves its extent
  -> frame allocation excludes that extent
  -> initial user-space preparation validates complete resource identity
  -> later bounded loader refinement may consume a named executable payload
```

The first x86-64 qualification uses QEMU's Linux `-initrd` handoff and the
Linux boot-protocol ramdisk fields. The adapter publishes an uncompressed,
deterministic `newc` CPIO archive containing one `init` file whose bytes are
the provider's currently sealed ELF image. The x86 provider validates the
32-bit and extended ramdisk fields, half-open extent, initial identity-map
bounds, exact archive length, and every archive byte. These are x86 adapter
and provider facts, not portable semantics.

## Ownership, evidence, and failure

The boot resource remains immutable and provider-private for this increment.
It is not allocator-owned, cannot be released as frames, and remains reserved
through the initial image qualification. The published boot provenance binds
the resource format, archive and payload sizes, and SHA-256 identities. QEMU
evidence binds the exact supplied file and records an ordered
`TOPAL_KERNEL_BOOT_RESOURCE_OK` witness after resident-image readiness and
before user entry.

Structural evidence must prove that:

- the boot adapter publishes one resource atomically with the kernel image;
- boot-memory and frame selection reject malformed metadata and never return
  an extent overlapping the resource;
- user-address-space preparation receives only the retained opaque handoff and
  validates the complete deterministic resource;
- source contains no ramdisk address, CPIO field, ELF byte, register, or
  instruction spelling; and
- all mismatches reach only the existing fatal disposition.

This increment does not load bytes from the resource, parse arbitrary CPIO or
ELF input, select paths in source, reclaim initramfs memory, support compressed
archives, relocations, interpreters, dynamic linking, PIE, ASLR, demand paging,
or a general filesystem.

## Cross-architecture pressure

The portable contract is one bounded immutable boot resource retained as a
reservation and required by a semantic initial-image layout. It does not
prescribe Linux boot parameters, CPIO, address width, archive transport, or
firmware conventions.

An AArch64 provider could derive the same authority from a Device Tree
`linux,initrd-start`/`linux,initrd-end` pair, while a UEFI provider could use a
loaded-image or firmware-volume object. Both must establish the same bounds,
immutability, reservation, and exact-identity obligations before the portable
layout transition succeeds. Capability architectures must retain a bounded
read capability rather than reconstructing authority from an integer.

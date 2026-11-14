# Step 4 provider-neutral initial boot resource

## Outcome

The x86-64 boot publication now contains a deterministic external
`initramfs.cpio` beside the generated `bzImage` and provenance record. QEMU
supplies it through the Linux ramdisk handoff. The generated provider validates
its bounded native metadata during boot-memory refinement, excludes its
physical interval from frame selection, and validates every resource byte
before preparing the initial user address space.

Portable Topal source observes only `TOPAL_KERNEL_BOOT_RESOURCE_OK` between
resident-image readiness and user-context preparation. It names no ramdisk
field, address, CPIO record, ELF byte, register, or instruction. This is a
systems-feature refinement and changes no core-language rule.

## Model and x86-64 lowering

The checked model requires exactly one boot-resource witness while both
finalized image-role mappings remain resident. A missing, renamed, duplicate,
early, or late witness is rejected, and user-context preparation cannot occur
before it.

Boot-resource schema `topal.boot-resource.x86_64-initial-newc/1` publishes a
1,088-byte deterministic uncompressed `newc` archive. Its single executable
member is `init`, containing the existing 848-byte sealed ELF image. Boot
adapter revision `/7` records both byte counts and SHA-256 identities and
atomically publishes exactly the boot image, resource, and provenance.

Provider revision `/42` validates the Linux boot-protocol base and extended
ramdisk fields, nonempty half-open extent, initial identity-map bounds, exact
size, CPIO identity, and complete archive contents. Physical-frame and
translation-backing selectors prove returned extents do not overlap the
resource. Initial address-space preparation receives only the retained opaque
handoff and repeats complete identity validation before mutating user backing.
Every mismatch fails closed.

Artifact revision `/43` records 62 placements and a 190-transition trace. Root
revision `/43` has 1,529 typed relocations, including 1,343 UART writes. No new
provider symbol or source-visible machine operation was introduced.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 42 ordered markers. Both supplied the published resource with
QEMU `-initrd`, observed the boot-resource witness before user entry, retained
all prior compatibility outcomes, remained in the live fatal halt, and left
serial output quiescent. Gate schema `/45` records:

- linked kernel: `abd154ca07b412d983d4e51d964b20c7f78638c950867c53ded5d20a115debb5`;
- boot image: `54c5f8fb125394cde70e603b4038916f5980b2005c95feecc904482855c7c1e8`;
- boot resource: `14114b99c2cb4575c452f855e7c0d02a4af3c0318a2bb3c87d941c37fa25e418`;
- initial user image: `5220ed61a0ed2120edc6238ef7b651ca18fdd7ed3cb3de74ecdd5e33a27b5491`;
- artifact provenance: `60f56f64348d8810bf321dee917baa8d68743941eea96a01831d4926d7adf3af`;
- boot provenance: `1cfac09267c9848f5b03c89770a5f792d4010b810fe8f5f360eaa6651cad30d5`;
- serial: `ea6e2208e5e3a82ab5de802bce9bb325e47c57567a6f81f18b8f77bcac4b917d`;
- evidence record: `3ad69304557b0a6f5751972c2da0e3e7f1f7d16fbc4e36f9dea5d5715edfb5eb`.

## Risk and review

Risk is medium: malformed native metadata or an incorrect generated comparison
could corrupt early boot or admit allocator overlap. Review therefore covered
authority-order consistency, deterministic archive construction, atomic
publication, field and arithmetic bounds, full-byte comparison, disjointness
branches, structural artifact assertions, negative semantic tests, full
workspace validation, and two independent physical runs.

## Remaining boundary

The provider still constructs the resident mapping from its sealed copy after
validating the identical external bytes. The next increment must discover the
bounded `init` member and populate the executable mapping from that resource.
General CPIO traversal, arbitrary ELF parsing, relocations, interpreters,
dynamic linking, BSS construction, PIE, ASLR, demand paging, and a filesystem
remain excluded.

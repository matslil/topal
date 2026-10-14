# Step 4 external ELF payload authority

## Outcome

The immutable boot resource is now the runtime authority for executable
payload bytes. The generated provider validates only the sealed archive
envelope, member identity, 256-byte ELF metadata prefix, and terminal record;
it no longer embeds the 592-byte executable payload in resource admission or
post-copy mapping verification.

Installation still performs 106 bounded resource reads and mapping writes.
After write revocation, verification performs another 106 resource reads and
compares them to every mapping qword before execute permission is published.
Portable source observes only the ordered
`TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK` witness. This systems-feature
refinement changes no core-language rule.

## Model and x86-64 lowering

The checked model requires the external-copy witness after external-load and
ELF-identity witnesses and before data, transfer, syscall, retention, or
teardown observations. Missing, renamed, duplicate, early, and late witnesses
are rejected.

Provider revision `/44` admits the existing bounded archive and ELF layout,
but excludes the executable payload interval from immediate byte comparisons.
Both installation and verification derive the same resource extent from the
retained Linux ramdisk handoff. The former copies into the allocator-backed
RW/NX construction mapping; the latter compares the now-RO/NX mapping back to
that extent before the existing RO/X promotion. Any changed source, incomplete
copy, destination mismatch, missing handoff, or metadata mismatch fails closed.

Artifact revision `/45` records 62 placements and a 192-transition trace. Root
revision `/45` has 1,604 typed relocations, including 1,418 UART writes. Boot
resource schema `/1` and boot adapter `/7` remain unchanged.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 44 ordered markers. Both loaded and verified the executable
against the external resource, completed every prior process-entry and Linux
syscall check, observed the new copy witness, remained in the live fatal halt,
and left serial output quiescent. Gate schema `/47` records:

- linked kernel: `1170ec904c16d8e0043120e01caa9f4c2cdac41ef40bf0819c97ee65121a0a8b`;
- boot image: `eba16ab098b4e179f789896f2ddf12138aa804ee1dfb92ba6f06d8bbd68c1592`;
- boot resource: `14114b99c2cb4575c452f855e7c0d02a4af3c0318a2bb3c87d941c37fa25e418`;
- initial user image: `5220ed61a0ed2120edc6238ef7b651ca18fdd7ed3cb3de74ecdd5e33a27b5491`;
- artifact provenance: `6d41ce00f098a13858f5e570c06cd851a877bd203479413aa380ffed4bdeecc3`;
- boot provenance: `92adfae5dd3c7ac9c8cc2cf4d877356049ba7a2080bd14e6d369e83af11dc1ac`;
- serial: `334274d87d74ba98df34e86c18df9c82fc98ded50f741a6d32e85cef3ec98967`;
- evidence record: `3cd3340af1b34d6a36aea9110437f7af2edc946aace73a18e8150b1e7d5338cf`.

## Risk and review

Risk is high because runtime acceptance no longer uses a second embedded
payload as an equality oracle. Review therefore covered the trusted-boot
boundary, exact metadata/payload separation, same-resource provenance across
copy and verification, complete extent counts, W^X ordering, immutable-source
assumption, generated instruction confinement, negative semantic tests, full
workspace validation, and two independent physical runs.

## Remaining boundary

Archive and ELF metadata are still exact rather than generically parsed. The
next loader increments must validate variable bounded metadata fields before
adding BSS, multiple load segments, or relocatable images. Authentication is a
separate board/security profile. Multiple files, compression, interpreters,
dynamic linking, PIE, ASLR, demand paging, and a general filesystem remain
excluded.

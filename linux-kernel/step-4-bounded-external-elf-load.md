# Step 4 bounded external ELF load

## Outcome

The initial executable mapping is now populated from the `init` member of the
validated external boot resource. The generated x86-64 installation operation
performs 106 bounded qword reads from that immutable member and 106 qword
stores into the allocator-backed writable construction mapping before the
existing read-only verification and executable-promotion lifecycle.

Portable Topal source still selects only the semantic `initial-probe` layout.
It observes `TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK` after the resident ELF
identity witness and before the existing data and syscall witnesses. It names
no archive field, filename, offset, ELF identity, address, register, or
instruction. This systems-feature refinement changes no core-language rule.

## Model and x86-64 lowering

The checked model requires exactly one external-load witness after credential
and ELF identity validation, while both finalized image roles remain resident.
A missing, renamed, duplicate, early, or late witness is rejected, and later
data, transfer, syscall, retention, and teardown observations depend on it.

The deterministic boot resource retains schema `/1`, a 1,088-byte archive, and
an 848-byte `init` member. Provider revision `/43` repeats handoff bounds and
schema validation, derives the member extent at the qualified archive boundary,
and copies its complete qword-aligned contents into the existing code-role
mapping. The installation operation contains resource-load and mapping-store
sequences rather than immediate executable qwords. Full resource validation
before construction and complete post-copy mapping verification remain in
force, and all failures remain fatal before execute permission publication.

Artifact revision `/44` records 62 placements and a 191-transition trace. Root
revision `/44` has 1,564 typed relocations, including 1,378 UART writes. The
boot adapter remains `/7` because the published resource format is unchanged.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 43 ordered markers. Both copied the executable from the QEMU
`-initrd` resource, entered at its validated ELF entry, completed the existing
initial-stack and Linux syscall checks, observed the external-load witness,
remained in the live fatal halt, and left serial output quiescent. Gate schema
`/46` records:

- linked kernel: `768ee301a47cb1f92099ce24feaba3cc02e34604c50cb1cbd458ce13ec0cf0b4`;
- boot image: `1413992f8cadc0815aaa28fe8e3ed2867420e68c9084529dd2505ce0e24b669b`;
- boot resource: `14114b99c2cb4575c452f855e7c0d02a4af3c0318a2bb3c87d941c37fa25e418`;
- initial user image: `5220ed61a0ed2120edc6238ef7b651ca18fdd7ed3cb3de74ecdd5e33a27b5491`;
- artifact provenance: `d132568ef75f9820d2d1af8c03488d3ebb8656ebdb63af0c947886e9e106b574`;
- boot provenance: `bcebec412fbe28ed0e749a0ab9e5da8cc98e11f827d678c9db8aa35755fb5504`;
- serial: `b481ae3a1f2cedd4b68c0239b30c037bb991c2e3d016a508c85870083727e949`;
- evidence record: `f9edf4e6e80939394b418bb0ad7bdfbcbadd24463c2441c1d958ddec9983cf31`.

## Risk and review

Risk is medium-high because executable bytes now cross an external-input to
user-executable boundary. Review covered immutable resource authority, archive
and ELF bounds, exact load extent, source/destination separation, W^X ordering,
complete post-copy verification, generated instruction confinement, semantic
ordering and negative tests, full workspace validation, and two independent
physical runs.

## Remaining boundary

The archive and ELF identities remain exact and provider-qualified. The next
loader work must replace exact whole-resource identity with independently
bounded field parsing and then add one explicitly designed capability at a
time: BSS, multiple load segments, or relocatable images. Multiple files,
arbitrary names, compression, `PT_INTERP`, dynamic linking, PIE, ASLR, demand
paging, filesystem lookup, and a general process model remain excluded.

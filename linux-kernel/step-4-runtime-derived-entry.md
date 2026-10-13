# Step 4 runtime-derived ELF entry

## Outcome

The external ELF header now supplies the initial user instruction pointer and
the matching `AT_ENTRY` process fact at runtime. The provider no longer uses a
separate embedded address when it constructs the initial stack or enters
ring 3, and constructed code compares `AT_ENTRY` with the resident ELF header.

Portable source observes only the ordered
`TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK` witness. ELF offsets, numeric
addresses, state layout, stack encoding, registers, and entry instructions
remain provider details. This systems-feature refinement changes no
core-language rule.

## Model and x86-64 lowering

The checked model requires the runtime-entry witness exactly once after the
runtime executable-extent witness and before runtime data/BSS, transfer,
syscall, retention, or teardown observations. Negative tests reject a missing
or renamed witness; the ordered continuation rejects early, duplicate, or late
placement.

Provider revision `/48` excludes `e_entry` from the immediate metadata oracle,
loads it from the same immutable ELF resource, and requires it to be at or
above the sealed executable payload floor and strictly below the validated
`code_start + p_filesz` bound. The selected entry is retained in opaque user
state, written into the initial stack's `AT_ENTRY` slot, loaded for `iretq`,
and compared by constructed code with the resident ELF header. A missing,
underflowing, end-equal, out-of-extent, or inconsistent entry fails closed.

Artifact and root revisions `/49` record 62 placements, a 196-transition
trace, and 1,754 typed relocations, including 1,568 UART writes. Boot resource
schema `/1` is unchanged; boot adapter revision `/10` binds the revised
provider and artifact.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 48 ordered markers. Both selected the external ELF entry,
constructed and consumed the matching `AT_ENTRY`, began user execution through
that selected value, completed every prior segment, process-entry, and Linux
syscall check, remained in the live fatal halt, and left serial output
quiescent. Gate schema `/51` records:

- linked kernel: `e69faabd8288482d2911d090efb5f11e0e56211232b822978c8e016db5f52f85`;
- boot image: `15ea7c7a9b75657ecd0decceb2e5c078fa4e92b02b93f970ffb50c58b30f9fc6`;
- boot resource: `2b8238ed1178facff10cd6ddc5af26e0d852682a3a3de32382cab82a587eb5b5`;
- initial user image: `b230a105144d38f41a71cb7e15d47535561067ac96b3a66bb44f4ef6000b4683`;
- artifact provenance: `7956ded7ca2162fc91b08662613729ff38492d5e755405dd7d2800cf8474c809`;
- boot provenance: `7995c0bac1532af43584694d203302155bef84cfb7fd9225dfe81f07e0c9a6eb`;
- serial: `766fbc7afcb2f58f2a6d84e1e1cd4dd0d048ecd288c11869e70a3753ddf12317`;
- evidence record: `c71647931d9f41c2d9b97672cdf351fb9f9ed60b5b850e3c81f7e7a91c8bcd9a`.

## Risk and review

Risk is high because a resource-controlled value becomes the first user
instruction pointer. Review covered metadata-oracle removal, lower and strict
upper extent bounds, one retained authority, stack/entry consistency,
resident-header comparison, nonzero-state checks, fail-closed paths, negative
model tests, structural instruction tests, full workspace validation, and two
independent physical runs.

## Remaining boundary

Segment placement, resource size, immutability, the payload floor, and the
generated qualification contents remain sealed. The profile does not yet
admit writable data, arbitrary segments, relocations, interpreters, dynamic
linking, TLS, PIE, ASLR, demand paging, path lookup, or a general process
loader.

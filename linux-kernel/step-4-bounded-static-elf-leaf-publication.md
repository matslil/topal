# Step 4 bounded static ELF leaf publication

## Outcome

The executable x86-64 systems profile now consumes the exact live 503-frame
static-ELF payload and its disjoint metadata frame into one opaque inactive
construction. The provider clears the complete 512-entry table, constructs 503
adjacent present, user-readable, writable, and non-executable leaves in payload
ordinal order, leaves nine entries absent, publishes the inactive table with a
memory fence, and independently verifies every entry before returning success.

The checked source observes the construction through
`TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK`, requests complete revocation,
releases the returned metadata frame, reclaims the payload, and releases the
whole payload extent. Revocation first verifies the live leaf policy, clears all
512 entries, publishes the clear state, and verifies that the complete table is
absent. This increment does not attach the construction to an active address
space or copy application bytes.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001` adds no core-language element. The
language checker admits only the exact payload and metadata identities, the
sealed `bounded-static-elf-initial-rw-nx` layout, the exact witness, and ordered
metadata revocation/release followed by payload reclaim/release. Negative cases
reject another layout, witness, owner, role, release identity, or incomplete
affine lifecycle. The semantic model represents live construction,
metadata-returned, fully revoked, payload-returned, and released states.

Root inspection proves exact provider dependencies, checked failure edges,
retained opaque bases, and ordered state transitions. Provider-object
inspection proves the 503 and 512 loop bounds, exact NX/user/write/present leaf
bits, nine absent entries, publication fences, pre-revocation verification,
complete clearing, and post-revocation verification. The checked root contains
148 fatal edges, a 330-event semantic trace, 3,317 typed relocations, and 2,925
UART calls. Artifact `/111`, provider and provider-object `/103`, root `/102`,
boot adapter `/68`, and gate `/113` identify the executable contract.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 89
ordered observations with identical hashes. The new witness follows mapping-
resource ownership and precedes revocation and resource return, so it is
reachable only after the provider constructs and verifies the exact inactive
leaf table. Recorded hashes are:

- linked kernel: `c61dcffcbd3348d2b41790041d4d062da9ba0f961f2a3dfe7b8928a5948e1128`;
- boot image: `475fe9806ed86a1eff36a557aacda74b6f176f47ffc064d382f4652b833599da`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `f3b4719789bef44016c5edcdf146380c60ae154592e7ad2b773436a6ccdbff26`;
- boot provenance: `802c746e9200efd87c3ebce178b7fd4b6376b4446d2b9770157c2757b011c927`;
- serial output: `c1dd781256b01c961b5712b5ddc6749573c4d8a5df701a9dd5f95e4d24cb4228`;
- evidence: `ed96a0cc00fcbc0014cae4688706569c6443b0b51635c7fe67cc68d9bdf764cc`.

## Risk and remaining boundary

Risk is high because provider-generated paging entries can violate isolation,
execution policy, or affine ownership even when the table is inactive. Review
covered exact entry bits and counts, physical adjacency, table initialization,
publish ordering, independent verification, fail-closed revocation, complete
resource return, typed dependencies, structural tests, and duplicate physical
execution.

The table is still revoked before use. The next increment must populate the
payload deterministically from the approved page-realization plan while the
construction remains inactive. Final per-page W^X and RELRO permissions, TLS,
stack construction, entry publication, active address-space attachment, and
BusyBox startup remain later work.

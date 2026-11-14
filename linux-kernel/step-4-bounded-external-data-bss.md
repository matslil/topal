# Step 4 bounded external data and BSS

## Outcome

The external initial ELF now carries a distinct immutable data segment and a
memory-only zero tail. The provider copies the complete initialized qword from
the validated boot resource, explicitly zeroes the following BSS qword,
revokes write authority, and then compares both qwords after RO/NX
publication. Constructed user code independently observes the initialized
sentinel and the zero tail before its existing syscall sequence.

Portable source observes the ordered
`TOPAL_KERNEL_USER_IMAGE_BSS_OK` witness between external-copy verification
and the prior data witness. Archive layout, ELF encoding, addresses, page-table
policy, qword operations, and zero-fill instructions remain provider details.
This is a systems-feature refinement and changes no core-language rule.

## Model and x86-64 lowering

The checked model requires the BSS witness exactly once after the external
payload-copy witness and before data, transfer, syscall, retention, or teardown
observations. Negative tests reject missing and renamed witnesses, while the
existing ordered continuation rejects early, duplicate, or late placement.

The initial ELF has three program headers: bounded RO/X code at `0x604000`, an
8-byte RO initialized extent at file offset 4096 and address `0x605000` with a
16-byte memory size, and the non-executable GNU-stack policy. `AT_PHNUM` is
three. The published ELF file is 4,104 bytes and its deterministic `newc`
resource is 4,344 bytes. Provider revision `/45` derives initialized data from
that resource and explicitly zeros BSS rather than relying on allocator state.

Artifact and root revisions `/46` record 62 placements, a 193-transition
trace, and 1,634 typed relocations, including 1,448 UART writes. Boot adapter
revision `/8` publishes the expanded resource; boot-resource schema `/1`
remains unchanged.

## Physical evidence

Two consecutive pinned `pc-q35-10.2`/`qemu64-v1` executions were byte-identical
and reached all 45 ordered markers. Both copied initialized data from the
external resource, observed an explicitly zeroed BSS tail after protection,
completed every prior process-entry and Linux syscall check, remained in the
live fatal halt, and left serial output quiescent. Gate schema `/48` records:

- linked kernel: `a405340ec641029c84f70642f24ff5a487d55242dc4202f1869d217eee04d906`;
- boot image: `be1ad7ac26c418ed964390d5776139e22cbe5677cb4999143710f21c1b388bd3`;
- boot resource: `fd2be78bfb6a99467fdd8d9b9d7c8cc0e3c07412c1b9122e8fee45a385d3fb2b`;
- initial user image: `079d7bcaca0d4cbfa91777ac8a15bf1eda1bf52f48116f2addcf8059c819407b`;
- artifact provenance: `df81d7797ea42e701a9c81b3f6525c2281906989f151a5e8622178b4a313e082`;
- boot provenance: `6bfb1e26d1611155054047df9e0637018f8433de8df6d60fda85efb9fe0f135d`;
- serial: `96d8c244cbaaddde730cd44c95d959d48827ced6bb6dab354af7075b821f6e92`;
- evidence record: `2163424dfc5ba1e1d37c2131ffe27a3b79d7b05b585a0de0aa427580204bf17a`.

## Risk and review

Risk is high because loader segment semantics, explicit initialization, W^X,
auxiliary-vector coherence, and user-visible execution meet at one boundary.
Review covered exact segment bounds and permissions, nonoverlap, same-resource
provenance, complete initialized and BSS extents, publication ordering,
post-protection verification, constructed-user observation, fail-closed
register use, negative model tests, structural instruction tests, full
workspace validation, and two independent physical runs.

## Remaining boundary

The ELF layout is still exact rather than generically parsed, and data remains
immutable. More load segments, arbitrary sizes and placements, writable data,
relocations, interpreters, dynamic linking, TLS, PIE, ASLR, demand paging, and
a general filesystem/process loader remain excluded.

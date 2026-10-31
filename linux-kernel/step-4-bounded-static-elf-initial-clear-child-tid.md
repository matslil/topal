# Step 4 bounded static ELF initial clear-child-TID word

## Outcome

The bounded static process now continues after FS-base installation through
one exact `set_tid_address(0x005f76d0)` registration. The clean-room payload
initializes that anonymous-heap word to one and verifies sealed TID one. On
terminal `exit_group(0)`, the provider zeros and verifies the retained word
while the process address space remains active, clears private registration,
and only then restores kernel state and revokes and erases the heap.

Rule `TOPAL-SYSTEMS-STATIC-ELF-INITIAL-CLEAR-TID-001` keeps syscall, address,
register, and task representation provider-private. Negative tests cover the
complete startup profile and ordered marker. Structural tests prove the
137-byte query/growth/FS/exit-word probe, native selector 218, exact candidate
and TID, pre-CR3 terminal zeroing, no futex wake, twenty-qword inverse clearing,
and preserved teardown.

Artifact `/161`, provider `/151`, provider object `/143`, root `/150`, boot
adapter `/69`, boot resource `/2`, and gate `/163` identify the contract. The
root has 408 operations, 469 semantic events, 8,796 relocations, and 6,174
console-writer relocations. The formal systems profile contains 153 rules.

Two independent pinned-QEMU runs reached all 159 observations and produced
identical evidence SHA-256
`94e802db2047d916f97ccbbdb4d96eec625d0465673ae9b5a1471cb9590bc1f3`:

- linked kernel `49b5a374e1950d9be64aa4d9a4817b4b25bedb572a45fc14f4528bfdaaef0336`;
- boot image `90dc10d347a4e127807e5e8e7a90dd357d64ee4bc97a47dd68164de691eb1669`;
- boot resource `5a881b692a517a6e2f7cbd97b0b15a542f78558ae9ebfaea01c06177b0ca2590`;
- static image `8a11530357a0bb551d67e930783c68f7cdb36bbcb2dcae69d532fa9a8f16e56a`;
- artifact provenance `21aa55b0b2d70f30adb01ea5223cb7c34061769ed8c0f3ce6db5016a0f8f1b61`;
- boot provenance `b46ce6b1ac91af52cfc2110194f7ae45d32b7d210f6b0a10d7182051dbb502a7`;
- source `e6a38712e11f72ba44a22dbec92c95cf0875971976232b66983876a934a1e873`;
- serial output `d66fadf30976a26952ac4096bb50b9100bbc38dbb6638859c31555cea2badcc4`.

Risk is high because terminal handling writes user memory during a privilege
transition. Review covers exact writable mapping authority, store-before-CR3
ordering, verification, private clearing, FS restoration, heap rollback,
fail-closed paths, structural checks, full workspace validation, and duplicate
physical execution.

This does not add futex wakeup, robust lists, cloning, multiple threads,
signals, BusyBox redistribution, or a prompt. The next pinned startup dependency
is the empty robust-list registration.

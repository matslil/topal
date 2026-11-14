# Step 4 bounded static ELF prepared-process retention

## Outcome

The qualified x86-64 path now transfers the complete random-slot-authorized
static ELF process into one opaque affine retained owner, closes the
construction-facing allocator channel without releasing its frame, publishes
`TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK`, restores the exact
construction and nested one-frame backing obligation, and publishes
`TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK`.

Both provider transitions completely validate the inactive hierarchy, exact
RW/NX stack leaf, guard, resident image and headers, deterministic process
table, future random-object zeros, fixed strings, and otherwise-zero stack.
They perform no store, fence, invalidation, release, or activation. Checked
source sees only the retained-process identity and sealed policies; resource
representation remains provider-private.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PREPARED-RETENTION-001` adds no core-language
element. Negative tests reject substituted owners, backing, value, policies,
and markers. The affine semantic model rejects duplicate, premature, or
out-of-order transitions and bootstrap completion while the retained owner is
live. Structural tests prove complete unchanged-state readback, both zero
random-object qwords, and the absence of stores, store fences, and active
address invalidation.

Artifact `/137`, provider and provider-object `/127`, root object `/126`, gate
`/139`, boot adapter `/69`, and boot-resource `/2` identify this contract. The
checked source contains 356 operations; the provider plan contains 128; the
root contains 190 fatal edges, a 417-event semantic trace, 5,306 typed
relocations, 4,830 UART calls, 90 required text symbols, and 131 observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions completed all
131 observations in order within the supported 60-second allowance and
produced identical hashes.

- static image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `0837b3434a3ff98870d08ba8458dc8a0537d116176dd88dc86827abe950faa11`;
- boot image: `2ca94710914a5dc816cb1aed84ac124404f02988062435bbcd0603965c8fe31e`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `5bc9bb45e8ae5e8f55af53d055d2b4c0d7c054521418b69fa0747090b39e4e78`;
- boot provenance: `3ba4a6f83adc4004b3e696cb15faa09409b48129389e2ab63a798b0de663f632`;
- serial: `16598910b013d2869bdc6dc7ac3afbfc0b0e6f1a33cb8495ff996440506855d3`;
- evidence: `dce773c05a63904d22e980f7d8a88e355a05ddbd340e5cc7aa8ad526f9eedb00`.

## Risk and remaining boundary

Risk is high because accidental frame release, aliasing, or mutation while the
allocator channel is closed would invalidate the later process. Review covered
exact owner and policy agreement, all retained identities and bytes, closed
allocator state, exact one-frame restoration, inactive translation state,
structural machine code, negative cases, and duplicate successful physical
execution.

This increment restores immediately as a qualification witness. It does not
yet carry the retained owner across the entropy pipeline, consume or expose
random material, publish `AT_RANDOM`, initialize TLS, construct a user context,
enter ring 3, execute BusyBox, or add a syscall.

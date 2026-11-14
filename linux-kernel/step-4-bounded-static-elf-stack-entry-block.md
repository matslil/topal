# Step 4 bounded static ELF stack-entry block

## Outcome

The qualified x86-64 path now constructs one minimal Linux-compatible process-
entry block in the guarded static ELF stack backing, publishes
`TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK`, completely clears the
frame, and publishes `TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK` before
leaf revocation. Checked source selects only the opaque owner, backing, sealed
`bounded-static-elf-busybox-true` profile, and complete extent; it receives no
address, pointer, word, alignment, offset, or byte-order representation.

The provider writes `argc` two, pointers to in-leaf `busybox` and `true`
strings, terminated `argv`, an empty terminated environment, and `AT_NULL` at
a 16-byte-aligned entry position. It reads back every semantic word and string
byte plus every otherwise-zero qword, then clears and verifies all 512 qwords.
The exact RW/NX leaf, guard, image leaves, inactive hierarchy, and current
bootstrap root remain unchanged.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-STACK-ENTRY-001` adds no core-language element.
Source-negative tests reject substituted owners, backing, profile, extent,
policy, or markers. The semantic model rejects early release, duplicate
publication or initialization, and clearing before initialization while
enforcing the published-leaf to initialized-entry to published-leaf ownership
cycle.

Structural provider tests prove the exact aligned entry offset, sealed pointers
and string words, complete initialization readback, complete 512-qword clearing
and readback, one fence per transition, preserved RW/NX leaf, and absence of
active-address invalidation. Artifact `/125`, provider and provider-object
`/115`, root object `/114`, boot adapter `/69`, boot-resource `/2`, and gate
`/127` identify this executable contract. The checked source contains 308
operations; the provider plan contains 104 operations; the checked root
contains 166 fatal edges, a 369-event semantic trace, 4,177 typed relocations,
3,749 UART calls, 66 required text symbols, and 107 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions initialized
and cleared the stack-entry block, reached both new witnesses in order,
completed all 107 observations, remained in the terminal fatal halt, and
produced identical artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `841ad795669c153e7bbafea6064ad003b5171997148c1fc52d14bc952753a3a4`;
- boot image: `f6080e36eed8ad07095080a1302aaec04f7b872e42232dfe7f53dc903306d796`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `435298e8259bb4f96897372ce48d5393a02ff65489892015ff0176ddd2308d66`;
- boot provenance: `e33ebe1c654be88a5b7a11801263ab56769e34754a1ce5ac1dcd29ea72252d3e`;
- serial output: `21cacef77124097de98baa64eeb12502285f44a47cec285c4e8f2bd38a3358ff`;
- evidence: `cfad75eb18265c296e3c2b1606ddc34fa09cc1974a1a4470d54c047055b8f32b`.

## Risk and remaining boundary

Risk is high because malformed pointers, alignment, termination, residual
bytes, or mutation of translation state could corrupt or disclose userspace
state. Review covered exact ABI facts and pointer relations, complete byte
coverage, target ordering, leaf/guard/image/hierarchy/root preservation,
affine rollback, failure edges, negative source tests, structural machine-code
inspection, and duplicate physical execution.

This increment clears the qualified block before returning. It does not retain
the process-entry block, add nonterminating auxiliary-vector entries,
initialize TLS, construct a user stack-pointer authority or context, enter
ring 3, execute BusyBox, or add a syscall.

# Step 4 bounded static ELF stack-backing zeroing

## Outcome

The qualified x86-64 path now completely zeros the reserved static ELF stack
frame, publishes `TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK`, and releases
the whole verified extent before slot and entry revocation. Checked source
selects only the opaque backing identity and complete extent; it receives no
address or clearing mechanism.

The provider validates a nonzero, 4096-byte-aligned backing identity below the
qualified physical ceiling, clears exactly 512 forward qwords, performs one
store fence, and compares exactly 512 forward qwords with zero. The checked root
retains the zeroed-backing affine state until whole release clears the backing
identity and restores the separately retained hierarchy identity.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-STACK-ZERO-001` adds no core-language element.
Source-negative tests reject substituted owners or backing, partial extents,
wrong witnesses, and slot revocation using the pre-zero owner. The semantic
model rejects release before zeroing and enforces slot-authorized-with-backing
to zeroed-backing to slot-authorized ownership.

Structural provider tests prove the complete forward clear, one store fence,
complete zero comparison, alignment and extent bounds, and no hierarchy access.
Artifact `/123`, provider and provider-object `/113`, root object `/112`, boot
adapter `/69`, boot-resource `/2`, and gate `/125` identify this executable
contract. The checked root contains 162 fatal edges, a 361-event semantic
trace, 3,981 typed relocations, 3,561 UART calls, and 103 ordered observations.

## Physical evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU executions zeroed and
released the stack backing, reached the new witness in order, completed all 103
observations, remained in the terminal fatal halt, and produced identical
artifact and serial hashes. Recorded hashes are:

- static user image: `0ca6467b2394c0c93327b27efcf40428add871819e12e06b76a276a693006ea9`;
- linked kernel: `15cd9d724f61e648e7624f6fab1703ec602ab0dde8a4bc03a36076e764071796`;
- boot image: `814943959f006ab805c5a634fbba026587900af8c77704fc3697dca86eb27b36`;
- boot resource: `4991f4cb505d69f5c052c23acb68e765fdab49e8d4eac50f1aad546ebfa6d3c1`;
- artifact provenance: `db2a01a1c19c6ca8896e8ca1a61224499f46ac807ccb180897fa7da12f9cc10f`;
- boot provenance: `584b070ece3bb7cbc0de992ff4ef6df0a17effb6a09e8d733a0b4f63c8934e22`;
- serial output: `7b195b0d9d55656ff3fe82c58c1fee438e5954c8db118ef6c24f6b90775d74bf`;
- evidence: `05f1bbdfdab6a566565b40b22c45c1eb019460802ea1ff764a75eafe58c69e0f`.

## Risk and remaining boundary

Risk is high because incomplete clearing or publication before initialization
could disclose recycled kernel data to userspace. Review covered complete byte
coverage, visibility ordering, alignment and bounds, private identities,
affine release order, failure edges, negative source tests, structural machine-
code inspection, and duplicate physical execution.

This increment releases the zeroed frame without mapping it. It does not
install a stack leaf, initialize `argc`/`argv`/`envp` or auxiliary-vector
contents, initialize TLS, construct a user context, enter ring 3, or execute
BusyBox.

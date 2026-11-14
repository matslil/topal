# Bounded static ELF entropy-generator lifecycle coexistence

This record resolves `TK-DEC-173` as one complete initialization, extraction,
and verified-destruction lifecycle for the initial entropy generator while the
static process remains behind its acquisition owner. It is a systems-profile
refinement only and adds no core-language element.

## Decision

After `TK-DEC-172`, `static image construction qualify entropy generator
lifecycle` shall consume and return the exact acquisition owner. It shall
accept only the sealed preceding entropy resources, generator identity one,
ChaCha20 algorithm identity one, output identity one, and policy
`bounded-static-elf-entropy-generator-lifecycle-coexistence`.

The provider shall completely revalidate the retained process and replay the
exact device, assessment, and credited-pool path. It shall initialize the exact
generation-one 32-byte ChaCha20 key from the 256-credit pool and perform the
exact first extraction: generation one to two, counter zero, nonce identity
zero, 64-byte block with only its final 32 bytes retained as opaque output
identity one. Neither source nor retained process may observe the key or output.

Before returning, the transaction shall reset and verify the transport,
overwrite and read back payload, ring, sample, pool, generator-key, and output
storage, reverse bus mastering, and restore gateway state. No entropy resource,
credit authority, key, output, byte, or DMA identity shall escape. Success
returns the same owner and permits
`TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK`.

## Architecture pressure test

Initialization and extraction are algorithm contracts rather than exposed
instruction sequences. X86-64, AArch64, and RISC-V may use different qualified
ChaCha20 realizations only if they preserve the exact generation, counter,
nonce, selected output range, key replacement, opacity, and verified
destruction semantics. Accelerated code may not leave additional live copies.

## Alternatives rejected

Publishing output directly to the process was rejected because generator
lifecycle and destination authorization are separate boundaries. Reusing the
pool as a key or exposing deterministic test vectors was rejected because
either would conflate affine resources or expose secret material. Returning a
live generator/output beside the unchanged acquisition owner or trusting wipe
without readback was rejected because it would weaken ownership or
confidentiality.

## Deliberate bounds

This increment admits one exact generator initialization, one first extraction,
complete destruction, and same-owner return. It rejects another identity,
algorithm, generation, counter, nonce, block selection, output, order,
duplicate, escaped authority or byte, partial erase, process mutation, or
recoverable disposition. It does not deliver `AT_RANDOM`, construct a user
context, enter the static process, or add a syscall.

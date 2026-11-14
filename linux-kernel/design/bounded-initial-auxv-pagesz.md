# Bounded initial `AT_PAGESZ` auxiliary vector

This record resolves `TK-DEC-054` as a systems-profile refinement. It extends
the provider-sealed Linux-compatible initial stack with one `AT_PAGESZ` entry
before `AT_NULL` and requires constructed code to validate and consume that
entry. It changes no core-language rule and exposes neither auxiliary-vector
encoding nor page size to source.

## Decision

The existing opaque address-space preparation and user-entry operations retain
their architecture-neutral meaning. For this bounded profile, the provider
must construct an aligned initial stack containing, in semantic order:

1. `argc` equal to one;
2. one `argv` candidate naming the sealed probe image and a null terminator;
3. an empty, null-terminated environment;
4. one `AT_PAGESZ` entry whose value is the provider's Linux base-page size;
5. a terminating `AT_NULL` entry; and
6. the immutable null-terminated `argv[0]` bytes within the same writable stack
   mapping.

The sealed constructed entry must validate the exact semantic contents,
including the page-size value used by the prepared address space, consume the
whole bounded block without retaining a candidate, restore the prior stack-top
position, and only then observe immutable data and transfer to the
compatibility probe.

After exact kernel return, source publishes the existing entry and stack
witnesses, then `TOPAL_KERNEL_USER_IMAGE_AUXV_OK`, followed by the existing
data, transfer, syscall, and residency witnesses. Both image-role owners remain
live through all observations and are subsequently revoked data first and code
second.

```text
prepare address space -> construct aligned bounded initial stack
  -> enter constructed code at stack block
  -> validate argc/argv/envp/AT_PAGESZ/AT_NULL/string
  -> consume bounded block -> restore stack top
  -> compatibility probe and returned constructed continuation
  -> exact kernel return -> entry -> stack -> auxv -> remaining witnesses
```

Source names neither auxiliary-vector tags nor values, pointer width, stack
direction, alignment, word order, addresses, padding, string bytes, nor load
encoding. Structural evidence must prove exact bounded initialization,
provider-consistent page size, entry stack selection, ordered constructed
validation and consumption, subsequent stack balance, and fail-closed
branches.

## Isolation and failure contract

The auxiliary vector and string lie only in the existing RW/NX user-stack
mapping. The immutable data mapping remains RO/NX and the code mapping remains
RO/X. The constructed continuation cannot select another tag, page size,
candidate, environment, string, layout, or auxiliary-vector order and cannot
retain a candidate after consumption.

Wrong alignment, tag, value, order, termination, argument contents, stack
balance, early or missing auxiliary-vector witness, mutation after
verification, or teardown before all witnesses is invalid. Fatal-only
completion remains in force after permission publication begins.

## Cross-architecture pressure

The portable contract is one provider-sealed `AT_PAGESZ` semantic entry whose
value agrees with the Linux base-page size of the prepared address space. It
does not prescribe the numeric tag, word width, byte order, alignment, stack
direction, address, or instruction encoding.

The initial x86-64 Q35 provider uses eight-byte words, the Linux x86-64
`AT_PAGESZ` tag, and a 4096-byte base page. AArch64 and RISC-V providers must
use their own Linux process-entry word conventions and configured base-page
size. Capability architectures must derive bounded stack and string
capabilities without fabricating or widening authority. In every case, source
observes only successful exact kernel return.

This increment does not admit source-selected auxiliary vectors, executable or
credential metadata, random state, platform strings, vDSO/vvar, dynamic
linking, arbitrary payloads, relocations, ELF loading, arbitrary segments or
aliases, demand paging, mapping ownership transfer, or a general process
model.

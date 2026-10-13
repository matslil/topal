# Bounded initial `AT_ENTRY` auxiliary-vector fact

This record resolves `TK-DEC-055` as a systems-profile refinement. It extends
the provider-sealed Linux-compatible initial stack with one semantic
executable-entry fact after `AT_PAGESZ` and before `AT_NULL`. Constructed code
must validate that the fact names its own initial instruction. It changes no
core-language rule and exposes neither auxiliary-vector encoding nor an
address to source.

## Decision

The existing opaque address-space preparation and user-entry operations retain
their architecture-neutral meaning. For this bounded profile, the provider
must construct an aligned initial stack containing, in semantic order:

1. `argc` equal to one;
2. one `argv` candidate naming the sealed probe image and a null terminator;
3. an empty, null-terminated environment;
4. one `AT_PAGESZ` fact consistent with the prepared address space;
5. one `AT_ENTRY` fact naming the first instruction selected by user entry;
6. a terminating `AT_NULL` entry; and
7. the immutable null-terminated `argv[0]` bytes within the same writable stack
   mapping.

The sealed constructed entry must validate the exact semantic contents and
must establish that the entry fact denotes the instruction at which it began.
It then consumes the whole bounded block without retaining a candidate,
restores the prior stack-top position, and continues through the existing
image and compatibility qualifications.

After exact kernel return, source publishes the existing entry, stack, and
page-size witnesses, then `TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK`, followed by
the existing data, transfer, syscall, and residency witnesses. Both image-role
owners remain live through all observations and are subsequently revoked data
first and code second.

```text
prepare address space -> construct aligned bounded initial stack
  -> enter constructed code at the sealed entry point
  -> validate argc/argv/envp/AT_PAGESZ/AT_ENTRY/AT_NULL/string
  -> prove AT_ENTRY denotes the current constructed entry
  -> consume bounded block -> restore stack top -> remaining qualifications
  -> exact kernel return -> entry -> stack -> page size -> entry fact
  -> remaining witnesses -> ordered teardown
```

Source names neither the entry address nor auxiliary-vector tags and values,
pointer width, stack direction, alignment, word order, padding, string bytes,
or load encoding. Structural evidence must prove exact initialization,
agreement between entry selection and the semantic entry fact, constructed
validation and consumption, subsequent stack balance, and fail-closed
branches.

## Isolation and failure contract

The auxiliary vector and string lie only in the existing RW/NX user-stack
mapping. The named entry remains within the finalized RO/X code-role mapping;
the immutable data mapping remains RO/NX. The constructed continuation cannot
select another tag, entry point, candidate, string, layout, or vector order and
cannot retain a candidate after consumption.

Wrong alignment, tag, value, order, termination, entry-point agreement,
argument contents, stack balance, early or missing entry-fact witness,
mutation after verification, or teardown before all witnesses is invalid.
Fatal-only completion remains in force after permission publication begins.

## Cross-architecture pressure

The portable contract is one provider-sealed semantic entry fact that denotes
the first instruction of the executable image selected by user entry. It does
not prescribe the numeric tag, pointer width, byte order, alignment, stack
direction, address, relocation model, or instruction encoding.

The initial x86-64 Q35 provider uses eight-byte words, the Linux x86-64
`AT_ENTRY` tag, and its sealed code-role entry address. AArch64 and RISC-V
providers must use their Linux process-entry conventions and their own sealed
entry point. Capability architectures must derive an executable entry
capability without fabricating or widening authority. In every case, source
observes only successful exact kernel return.

This increment does not admit source-selected auxiliary vectors, ELF program
headers, credentials, random state, platform strings, vDSO/vvar, dynamic
linking, arbitrary payloads, relocations, general ELF loading, arbitrary
segments or aliases, demand paging, mapping ownership transfer, or a general
process model.

# Bounded initial `AT_EXECFN` auxiliary-vector fact

This record resolves `TK-DEC-056` as a systems-profile refinement. It extends
the provider-sealed Linux-compatible initial stack with one semantic
executable-name fact after `AT_ENTRY` and before `AT_NULL`. Constructed code
must validate that this pointer-bearing fact names the same bounded immutable
string as `argv[0]`. It changes no core-language rule and exposes neither an
auxiliary-vector encoding nor a user address to source.

## Decision

The existing opaque address-space preparation and user-entry operations retain
their architecture-neutral meaning. For this bounded profile, the provider
must construct an aligned initial stack containing, in semantic order:

1. `argc` equal to one;
2. one `argv` candidate naming the sealed probe image and a null terminator;
3. an empty, null-terminated environment;
4. provider-consistent `AT_PAGESZ` and executable-entry `AT_ENTRY` facts;
5. one `AT_EXECFN` fact naming the same immutable string as `argv[0]`;
6. a terminating `AT_NULL` entry; and
7. the immutable null-terminated executable-name bytes within the same writable
   stack mapping.

The sealed constructed entry must validate the exact semantic contents and
establish pointer identity between `argv[0]` and `AT_EXECFN` before validating
the shared string. It then consumes the whole bounded block without retaining
a candidate, restores the prior stack-top position, and continues through the
existing image and compatibility qualifications.

After exact kernel return, source publishes the existing entry, stack,
page-size, and entry-fact witnesses, then
`TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK`, followed by the existing data,
transfer, syscall, and residency witnesses. Both image-role owners remain live
through all observations and are subsequently revoked data first and code
second.

```text
prepare address space -> construct aligned bounded initial stack
  -> enter constructed code -> validate scalar process-entry facts
  -> prove argv[0] == AT_EXECFN -> validate their shared bounded string
  -> validate AT_NULL -> consume block -> restore stack top
  -> remaining qualifications -> exact kernel return
  -> ordered stack, auxiliary-vector, image, and teardown witnesses
```

Source names neither the executable-name address nor auxiliary-vector tags and
values, pointer width, stack direction, alignment, word order, padding, string
bytes, or load encoding. Structural evidence must prove exact initialization,
alias agreement, in-block termination, constructed validation and consumption,
subsequent stack balance, and fail-closed branches.

## Isolation and failure contract

The auxiliary vector and shared string lie only in the existing RW/NX
user-stack mapping. The named code entry remains within the finalized RO/X
code-role mapping; immutable image data remains RO/NX. The constructed
continuation cannot select another tag, candidate, string, layout, or vector
order and cannot retain a candidate after consumption.

Wrong alignment, tag, pointer identity, bounds, termination, string contents,
order, stack balance, early or missing executable-name witness, mutation after
verification, or teardown before all witnesses is invalid. Fatal-only
completion remains in force after permission publication begins.

## Cross-architecture pressure

The portable contract is one provider-sealed executable-name fact that aliases
the same bounded immutable string as `argv[0]`. It does not prescribe the
numeric tag, pointer width, byte order, alignment, stack direction, address,
pointer representation, or instruction encoding.

The initial x86-64 Q35 provider uses eight-byte words, the Linux x86-64
`AT_EXECFN` tag, and one canonical user virtual address for both facts.
AArch64 and RISC-V providers must use their Linux process-entry conventions.
Capability architectures must derive both bounded string capabilities from the
same authority without fabricating or widening bounds. In every case, source
observes only successful exact kernel return.

This increment does not admit source-selected arguments, executable names or
auxiliary vectors, ELF program headers, credentials, random state, platform
strings, vDSO/vvar, dynamic linking, arbitrary payloads, relocations, general
ELF loading, arbitrary segments or aliases, demand paging, mapping ownership
transfer, or a general process model.

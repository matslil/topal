# Bounded Linux-compatible initial user stack

This record resolves `TK-DEC-053` as a systems-profile refinement. It requires
the prepared user execution context to enter with one provider-sealed,
Linux-compatible initial stack block and requires constructed code to validate
and consume that block before its existing data and syscall qualifications. It
changes no core-language rule and does not expose general stack construction,
addresses, words, layouts, strings, or auxiliary-vector policy to source.

## Decision

The existing opaque address-space preparation and user-entry operations retain
their architecture-neutral meaning. For this bounded profile, the provider
must construct an aligned initial stack containing:

1. `argc` equal to one;
2. one `argv` candidate naming the sealed probe image and a null terminator;
3. an empty, null-terminated environment;
4. a terminating `AT_NULL` auxiliary-vector entry; and
5. the immutable null-terminated `argv[0]` bytes within the same writable stack
   mapping.

The entry context begins at that block. After the compatibility probe returns,
the sealed constructed continuation must validate the exact semantic contents,
consume the bounded block without retaining a candidate, restore the prior
stack-top position, and continue through its existing data, output, input,
identity, and terminal qualifications.

After exact kernel return, source publishes the entry witness,
`TOPAL_KERNEL_USER_IMAGE_STACK_OK`, and then the existing data, transfer,
syscall, and residency witnesses. Both image-role owners remain live through
all observations and are subsequently revoked data first and code second.

```text
prepare address space -> construct aligned bounded initial stack
  -> enter user at stack block -> compatibility probe preserves stack
  -> constructed continuation validates argc/argv/envp/AT_NULL/string
  -> consume block and restore stack top -> existing qualifications
  -> exact kernel return -> entry -> stack -> data -> transfers -> residency
```

Source names neither pointer width, stack direction, alignment, word order,
addresses, padding, string bytes, nor auxiliary-vector encoding. Structural
evidence must prove exact bounded initialization, entry stack selection,
constructed validation and consumption, preservation across the compatibility
probe, subsequent stack balance, and fail-closed branches.

## Isolation and failure contract

The initial block and string lie only in the existing RW/NX user-stack mapping.
The immutable data mapping remains RO/NX and the code mapping remains RO/X.
The constructed continuation cannot select another argument count, candidate,
environment, auxiliary vector, string, size, or layout and cannot retain a
candidate after consumption.

Wrong alignment, missing termination, an out-of-block candidate, wrong
argument count or bytes, nonempty environment, nonterminating auxiliary
vector, stack imbalance, early or missing stack witness, mutation after
verification, or teardown before all witnesses is invalid. Fatal-only
completion remains in force after permission publication begins.

## Cross-architecture pressure

The portable contract is an aligned, bounded Linux process-entry block with
semantic argument, environment, auxiliary-vector, and string contents. It does
not prescribe a pointer width, stack direction, alignment value, byte order,
padding, address, or load encoding.

An x86-64 provider may use eight-byte words and 16-byte entry alignment.
AArch64 and RISC-V providers must use their Linux process-entry alignment and
word conventions. Capability architectures must derive bounded stack and
string capabilities without fabricating or widening authority. In every case,
source observes only successful exact kernel return.

This increment does not admit source-selected arguments or environments,
general auxiliary-vector entries, random state, executable metadata, TLS,
vDSO/vvar, dynamic linking, arbitrary payloads, relocations, ELF loading,
arbitrary segments or aliases, demand paging, mapping ownership transfer, or a
general process model.

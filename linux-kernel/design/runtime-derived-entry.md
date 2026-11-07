# Runtime-derived ELF entry contract

This record resolves `TK-DEC-065` as a systems-profile refinement. It makes
the validated external ELF header authoritative for the initial instruction
pointer and matching `AT_ENTRY` fact. It changes no core-language rule.

## Decision

The loader must read `e_entry` from the same immutable ELF resource used for
segment construction. It must prove that the address lies within the
validated executable file extent, refers to the sealed executable mapping,
and is representable by the target entry mechanism. The selected entry is
retained as provider-owned process state.

The provider must use that one selected value both when constructing the
initial stack's `AT_ENTRY` value and when entering user execution. Constructed
user code must compare `AT_ENTRY` with the resident ELF header rather than a
second provider-embedded address.

```text
validate code_start <= e_entry < code_start + code_filesz
  -> retain selected entry in opaque process state
  -> construct AT_ENTRY from selected entry
  -> enter user execution at selected entry
  -> verify stack AT_ENTRY == resident ELF e_entry
```

After the runtime executable-extent witness and before the runtime data/BSS
witness, source publishes `TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK`.
Portable source does not observe the address, ELF offset, register, stack
encoding, or target entry instruction.

## Failure and authority

A missing, underflowing, end-equal, out-of-extent, nonrepresentable, or
resource-incoherent entry fails before user execution. The executable mapping
must already contain the selected byte and must retain RO/X authority at
entry. Entry selection grants no additional mapping or write authority.

The initial sample retains its sealed placement and generated contents; this
increment removes the duplicate runtime entry authority but does not admit
arbitrary placement or arbitrary application payloads.

## Cross-architecture pressure

The common contract is selection of one validated address inside an
executable segment and consistent use of that selection for process metadata
and first instruction fetch. x86-64, AArch64, and RISC-V providers may differ
in canonical-address validation, entry-state construction, cache completion,
and exception-level transition. Those mechanics remain provider-private.

This increment does not admit writable data, additional segments,
relocations, interpreters, dynamic linking, TLS, PIE, ASLR, demand paging,
path lookup, or a general process loader.

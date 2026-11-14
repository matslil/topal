# Bounded ELF identity and program-header facts

This record resolves `TK-DEC-058` as a systems-profile refinement. It places
one provider-sealed ELF identity and program-header table in the existing
finalized code-role mapping, selects an entry instruction within that mapping,
and extends the initial auxiliary vector with matching `AT_PHDR`, `AT_PHENT`,
and `AT_PHNUM` facts. It changes no core-language rule and does not admit
source-selected files or a general ELF loader.

## Decision

The provider must construct one immutable executable image whose code-role
mapping begins with a valid target ELF header and a bounded program-header
table. The ELF entry field and user-entry operation must select the same first
instruction later in that mapping. The table describes the bounded executable
load mapping and its non-executable stack policy.

After the existing credential facts and before `AT_NULL`, the initial stack
must contain semantic program-header-address, entry-size, and entry-count facts
that exactly describe this resident table. Constructed entry code validates all
three facts, the existing terminator and string, and its own selected entry
point. It consumes the complete bounded block and restores the prior stack top
before continuing.

After exact kernel return, source publishes the prior process-entry witnesses,
then `TOPAL_KERNEL_USER_IMAGE_ELF_OK`, followed by the existing data, transfer,
syscall, residency, and teardown witnesses.

```text
construct sealed ELF identity + bounded program headers in code mapping
  -> finalize RO/X mapping -> construct matching initial PHDR facts
  -> enter at ELF entry -> validate PHDR address/size/count and prior facts
  -> consume stack -> remaining qualifications -> exact kernel return
  -> ordered ELF witness while code and data mappings remain resident
```

Source names neither ELF bytes, class, machine number, addresses, segment
flags, table sizes, counts, auxiliary-vector tags, nor load layout. Structural
evidence must prove header/table validity, agreement among the ELF entry,
selected entry, and auxiliary-vector facts, mapping bounds and permissions,
exact installation and verification, bounded stack consumption, and fail-
closed branches.

## Isolation and failure contract

The ELF identity, table, and generated executable payload share the existing
single RO/X code-role mapping. They cannot overlap the RW/NX stack or RO/NX
data mapping. The program-header table cannot grant writable-executable
authority, describe an unowned range, or select another entry.

A malformed class, byte order, machine, header size, entry, table offset,
entry size or count; inconsistent auxiliary-vector facts; permission mismatch;
stack imbalance; an early or missing ELF witness; or teardown before all
witnesses is invalid. Fatal-only completion remains in force after permission
publication begins.

## Cross-architecture pressure

The portable contract is a bounded executable identity, an immutable resident
program-header table, a selected entry within an executable mapping, and three
matching semantic process-entry facts. It does not prescribe ELF class,
machine number, byte order, address width, header representation, segment
flags, table shape, or instruction encoding.

The initial x86-64 provider uses ELF64 little-endian `EM_X86_64`, one
page-aligned executable load description, a non-executable GNU stack policy,
and Linux x86-64 auxiliary-vector encodings. AArch64 and RISC-V providers must
use their own ELF machine and Linux process-entry conventions. Capability
architectures must derive bounded executable and table capabilities without
fabricating authority.

This increment does not parse external ELF input, perform relocations, admit
interpreters or dynamic linking, map arbitrary segments, construct BSS,
support PIE/ASLR, accept source-selected entry points, expose program headers
to Topal source, or define a general loader or process model.

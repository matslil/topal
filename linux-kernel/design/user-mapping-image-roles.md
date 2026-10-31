# Sealed immutable user-mapping roles

This record resolves `TK-DEC-044` as a systems-profile refinement. It assigns
distinct immutable code and data roles to the two bounded mapping generations
without admitting persistent mappings or a general image loader. It changes no
core-language rule and exposes no address, permission bit, or descriptor.

## Decision

Generation one follows the approved W^X path through executable promotion and
ends as an opaque `ExecutableCodeMapping` with RO/X authority. Generation two
remains non-executable: `user mapping seal immutable data` consumes its exact
verified RO/NX refinement and returns an opaque `ImmutableDataMapping` with the
same RO/NX authority. Unmap consumes the role-specific final refinement.

```text
generation 1: RW/NX -> RO/NX -> verified -> ExecutableCode(RO/X) -> unmap
generation 2: RW/NX -> RO/NX -> verified -> ImmutableData(RO/NX) -> unmap
```

Code and data roles are not interchangeable. Source cannot execute-promote the
data generation, seal the code generation as data, unmap an unclassified
verified mapping, or reuse either consumed refinement.

## Role and ordering contract

Data sealing preserves provider, address-space, mapping, backing, placement,
memory kind, user accessibility, contents, read-only authority, and NX. It is
an authority classification rather than a target permission change. Success
requires the provider to prove the mapping still has the completed RO/NX state
produced by protection and verification.

Code promotion continues to require target translation and instruction-
observation completion. Data sealing must not perform or imply executable
publication. Neither role may restore write authority, and W+X is invalid.

## Alternatives and cross-architecture pressure

Promoting both generations to executable authority grants more authority than
immutable data needs. Leaving both as an undifferentiated verified mapping
hides future image policy. General source-selected segment flags would
prematurely define ELF, alias, relocation, and VFS policy.

The common role distinction is identical on x86-64, AArch64, and RISC-V.
Executable promotion retains each target's approved completion sequence; data
sealing validates that the target mapping remains read-only and non-executable
without requiring a permission update. Permission representation and any
validation mechanics remain provider-private.

The refinement still uses one frame, one sealed guard location, a fixed byte,
two sequential generations, and mandatory unmap. It does not admit concurrent
code/data mappings, persistence into user execution, arbitrary segment flags,
relocations, an image layout, or ELF loading.

This systems-only refinement is covered by the standing project approval for
autonomous changes that do not alter core-language meaning.

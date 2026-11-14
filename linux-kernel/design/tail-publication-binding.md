# Source and native binding for the tail root

This continues TK-DEC-282 under the approved systems-only direction. It closes
the publication boundary of the [kernel root adapter](tail-kernel-root.md).
It introduces no source address, assembly, core value or ownership semantics.

Tail publication must retain the root source and the complete selected ordinary
source/module snapshot. The checked program alone cannot supply the missing root
source. An explicit source-bearing publication entry reparses that source with
the existing target and affine checker. The program-only API continues to reject
the tail interface. The ordinary compiler pipeline already has the root text and
can use the source-bearing entry without ambient discovery or a new source form.

Boot packaging must reconstruct the checked root, source plan, bounds, LLVM,
native object closure, stack costs and target adapters. It uses the qualified
LLVM toolchain at packaging time, with the same closed compilation options.
This adds a packaging dependency for the new interface; the guest has no such
dependency. Serialized hashes, native objects, source plans and range assertions
cannot replace reconstruction. Other source-library profiles keep their existing
publication behavior.

The reconstruction uses static relocation and explicitly local binding of the
closed LLVM definitions and their imports. Dynamic interposition and GOT/TLS
relocations are outside this profile. A global dead-code pass removes unreachable
hosted runtime definitions from the ordinary source object before qualification;
it cannot justify writable source, runtime or controller storage. All retained
instructions, imports, data and stack paths still undergo native inspection. The
optimizer's temporary-path ModuleID comment is normalized so identical inputs
produce identical recorded LLVM bytes in separate packaging workspaces.

The linked comparison must cover all allocated sections of the root, provider,
ordinary source, private runtime, controller and both target adapters. This
includes anonymous constants, writable storage and zero-initialized storage.
Compiler-owned section endpoints identify each input's precise contribution
after linking. A fresh object's relocations are replayed against checked section
placements and unique external definitions, then the complete relocated bytes
are compared with the kernel. Unsupported relocations fail closed. Local symbols
resolve in their own freshly rebuilt object; a same-named symbol in another
object cannot substitute for them. Original section permissions, alignment,
defined-symbol placement and complete input domains are checked as well.
GNU retention, merging and string flags may propagate from another input into a
shared output section. They control input processing and grant no mapped
permission. That output-only propagation is excluded from section-flag equality;
exact endpoints, disjoint contributions and complete relocated bytes still prove
preservation of the fresh input. Actual load permissions remain an exact check.
The comparison also checks each contribution's actual load-segment projection,
including permissions and any initialized bytes overlapping zero storage. Matching
section-table bytes alone cannot certify the bytes that the boot adapter loads.
The ELF entry must select the reconstructed bootstrap entry.

Metadata uses a distinct tail-root schema and the actual rebuilt source/native/
caller facts. Boot rejects missing, null, duplicate, mixed, orphan, changed or
unreconstructable bindings. The initial owner remains the checked exclusive
64-byte plain bootstrap allocation, with facts `(64, 0, 1)`, actual true-result
gate, privileged masked context and complete normal/fatal retirement. Binding
the regenerated entire root and provider preserves that owner lifecycle and its
callsite instead of treating an address range as permission.

The kernel experiment's test-only boot helper is not an alternative publication
path. Canonical publication remains refused until reconstruction, complete linked
comparison, resource/domain checks and boot validation are integrated and tested.
This binding does not itself complete general allocator integration or Linux,
container and VM-host acceptance.

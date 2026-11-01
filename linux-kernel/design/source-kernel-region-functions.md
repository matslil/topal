# Checked source functions over borrowed kernel bytes

`TK-DEC-270`; systems-only refinement under standing authorization.

Implement the source and object boundary of
[borrowed kernel regions](borrowed-kernel-regions.md). Ordinary helper functions
must execute on the owner's bytes using shared function lowering, rather than
another native Linux policy or a second memory simulator. Kernel-root invocation,
artifact integration and QEMU reclamation remain subsequent qualification work;
a standalone source object is not a replacement-kernel capability.

## Source boundary

A qualified systems-library analysis context admits the sealed classifier
`KernelByteRegion`. Selecting `systems` grants its vocabulary; the qualified
caller supplies the live region. Source cannot construct a region from literals,
addresses, another ordinary value or a serialized object. An ordinary application
analysis does not acquire this classifier or machine operations.

The entry is one published ordinary `(region : KernelByteRegion) -> Unit`
function. Select it by checked declared signature with symbolic compiler inputs,
not by constructing a fake region in source or appending an executable client.
Existing shared parsing, module selection, visibility, instantiation, function
calls, arithmetic, decisions and Optional meanings remain authoritative.

The initial byte members are:

| Operation | Result and refusal |
| --- | --- |
| `region byte count` | Nat extent of this view |
| `region byte load (offset-bytes is index)` | Optional Nat; Some byte 0..255, None outside the view |
| `region byte store (offset-bytes is index, value is value)` | Boolean; false for missing write rights, an out-of-window index or a value above 255 |
| `region byte window (offset-bytes is start, byte-count is count, rights is read-only)` | Optional KernelByteRegion; a contained read-only view or None |
| the same window operation with `rights is read-write` | a contained writable view or None if the parent is read-only |

Indices, values and extents use ordinary arbitrary-precision Nat. A value too
large for native access produces the stated refusal; it is never truncated.
Empty end windows are valid and contain no readable byte. Derivation preserves
the same owner identity. Count and checked window derivation observe immutable
view metadata; byte reads/writes infer Read/Write of the owner's resource.
Named argument expressions evaluate once in source field order, before the
primitive performs its checks. Native argument slot order does not reorder
those expressions. A primitive refusal does not roll back effects already
performed by argument helpers.

Entry accepts one region of at most 65536 bytes. Helpers may receive explicit
views and admitted ordinary numeric/Boolean/Unit values, and return admitted
ordinary values. Region-containing returns, captures, persistent storage,
callbacks, recursion, suspension, concurrent access and ambient state remain
unqualified. Optional views may be consumed by a local decision; they cannot
escape through an aggregate, function result or native result. Views are loans,
not owning resources with a user-callable release operation.

## Effects and lifetime

The closed instantiated call graph is qualified before object publication.
Every view originates in the one entry owner, through parameters or checked
window derivation. Aliases therefore retain one abstract owner in inferred
Read/Write evidence, including transitive helper calls. The compiler must reject
a false empty `Effects ()` upper bound. Explicit nonempty source effect bounds may remain
an explicit initial-profile diagnostic; inference cannot silently classify a
byte operation as pure.

Function results and capture restrictions establish nonescape. Synchronous direct
calls establish caller suspension; callee-local views are unavailable after
return. Native transport may share a private representation only when this
preserves the model's owner, generation, bounds, rights, alias and lifetime
observations. Source cannot inspect transport identity. No source-visible
mutable reference, allocator construction or core error policy is introduced.

## Native object and caller obligations

The x86-64 object uses ordinary shared function lowering with real helper frames
and debug source/parameter information. A generated private adapter accepts an
explicit qualified owner/invocation binding. It validates code/generation,
extent/overflow, rights and active lifetime before access. Addresses originate
only in the caller's live region authority; numeric words or a C test label
cannot mint it. A native test caller supplies valid mapped control, region and
temporary extents; it is not an implementation of kernel ownership or entry.

Source execution uses an explicitly lent bounded temporary arena and qualified
allocation/fatal hooks. The object must contain no hosted allocator, syscall,
foreign runtime, writable global or uninspected machine interface. Native
layout, target register placement and pointer representation remain private.
All byte reads/stores and view metadata must be bounded before use. Source
temporaries and derived-view storage are charged along with ordinary shared
numeric/Optional algorithms. Prove complete stack/call/allocation/view bounds;
measured peaks are supplementary evidence. The TK269 graph/frame/view limits
remain in force, and undercharged bindings fail before publication.
The initial monotonic temporary realization additionally bounds all issued view
nodes, including retired nodes awaiting arena erasure, to 256 per invocation.
This conservative storage certificate is distinct from the model's live-view
limit. Ordinary numeric values retain the existing 256-bit proof budget; a
larger unsupported bound diagnoses rather than truncating values.

The caller must finish invocation cleanup before returning the owning resource
to ordinary use, including temporary metadata, arena, retired stack and code
bindings. Normal completion/cancellation preserves successful stores. The
standalone object phase must not claim physical kernel cleanup merely because
an independent native caller observes correct bytes or source frames.

AArch64 and RISC-V retain these same source and lifetime laws. Each needs its own
object/calling/representation/access and reclamation qualification. Only x86-64
object implementation is in scope now.

Risk is high at hidden lifetimes, representation/authority separation, effects,
ordinary lowering, complete bounds and publication. Review follows authoritative
resource/effect/layout meaning, the TK269 protocol, declared export selection,
every resource-producing/use path, complete reachable calls, native accesses,
malformed evidence and independent byte/caller/debug oracles. The implementing
agent performs these review roles; human PR review remains required.

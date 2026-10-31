# Bounded immutable numeric Arrays

`TK-DEC-259`; systems-only qualification adopted under standing authorization.

Qualify existing ordinary `List Int` construction, fixed `Array (N, Int)`
collection, checked constant-index lookup and `Optional Int` decisions in the
owned numeric invocation. This implements the Array/Optional meanings already
defined in `docs/containers.md`; it introduces no core type or byte authority.
The first source use transports the interval through an Array helper inside
the existing clock module. Native scalar input/result interfaces stay unchanged.

Admission retains exact finite sequence length and a separate numeric range for
each entry, with at most 256 entries. List construction charges every head and
tail evaluation and each node allocation. Collection charges the source once
and the actual Array header allocation. Lookup charges its receiver once and
the shared Optional result allocation, including out-of-bounds `None`.
Optional decisions retain possible presence and payload bounds, bind payloads
only on the `Some` path and charge the maximum reachable branch. Equal-length
sequence branch joins combine corresponding entry bounds; unqualified lengths,
elements, operations, recursion and effects reject publication. All generated
functions still undergo complete object-closure and stack inspection.

Use the shared ordinary collection lowering and runtime, retaining only the
qualified Array helpers after dead-code elimination. Bind reviewed collection
runtime bytes into the allocation certificate. No alternative kernel-only Array
algorithm is introduced. The existing 4096-byte stack and 65536-byte invocation
arena remain; all list nodes, headers, Optional wrappers and numeric payloads
are invocation-owned and must be erased before return. Source never receives a
native address, layout, context or allocator capability.

Qualification must cover actual imported Array arguments, independently decoded
entry values and length, valid and out-of-bounds access, empty Arrays, Optional
branch joins, complete evaluation costs, malformed/unqualified inputs and
resource exhaustion. Retain independent deadline arithmetic, Record-order
transport, changed-helper rejection, previous process/FIFO workloads and full
native-state/storage reclamation. AArch64 and RISC-V retain the semantic shape
but require independent private representation, runtime, calling convention and
stack qualification.

Risk is high at bounded traversal, per-entry range preservation, Optional branch
accounting, runtime closure and lifetime. This is a prerequisite for source
parsing of owned byte snapshots, not qualification of external-memory access,
general containers, persistent subsystem state, distribution boot or containers.

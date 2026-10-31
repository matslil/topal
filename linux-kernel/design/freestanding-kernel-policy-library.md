# Source-defined freestanding kernel policy library

Status: systems-only direction adopted under standing chat authorization;
implementation and qualification pending (`TK-DEC-252`).

Step 2 requires kernel policy to live in Topal libraries/modules over qualified
machine elements. The current bounded adapters still contain policy in native
backend code. This refinement first moves immutable boot-root open access policy
into ordinary Topal functions, using the existing shared frontend and ordinary
Boolean/Enum function lowering. It does not introduce source assembly, another
core function language or fabricated application output.

## Artifact and ownership boundary

The artifact may declare one `kernel-library` using
`lang systems source-library` with `source`, `export`, `interface` and
`stack-limit-bytes` fields. Source is a relative `.t` path below the explicitly
selected library root; export is a structural identifier naming a public
function in that unit. The initial semantic interface is `boot-open-decision-v1`
and the initial extra caller-stack limit is 1024 bytes. Missing, duplicate,
unselected, escaping-path, signature or interface inputs fail before publication.
The ordinary compiler `--library-root` selects the input tree; an in-memory
publication API must explicitly supply the matching source snapshot.

The source unit is checked by the ordinary shared language frontend. A concrete
unprivileged client instantiation selects the export's Boolean signature; its
whole reachable helper graph, rather than only one result path, is lowered.
The initial profile admits only Boolean and nominal Enum parameters/results,
local bindings, finite decisions and direct acyclic calls. Effects, resource or
address values, allocation, foreign calls, dynamic callable values, recursion,
module imports and other forms remain explicitly unqualified. Rejection is a
profile diagnostic, not a new result or new meaning for valid core Topal.

The artifact binds immutable compiled code and its checked interface. Static
process entry borrows that binding from the root, retains it across exec and
erases its private witness before terminal return. Native code pointers and
calling conventions remain implementation details. A source choice cannot grant
writable authority in the immutable filesystem; the native caller verifies its
owned rights before publishing a description. No application value selects a
privileged function or creates code authority.

## First kernel policy unit

`kernel/policy/boot-open-access.t` will define the public `BootOpenDecision`
Enum with semantic alternatives `ReadDescription`, `NotDirectory`,
`DirectoryWrite` and `ReadOnlyView`, plus public `boot-open-decision` and private
helpers. It accepts three Boolean facts: write intent, directory object and a
directory-only requirement. It first rejects a directory-only regular object,
then selects a directory-write/read-only-view denial or a read description.
It contains no native flag, selector, errno or address.

The interface fixes those semantic alternatives, not their source order or
native discriminants. A generated private adapter maps actual checked Enum
representation to the native provider's versioned semantic choices. The caller
continues normal flag/name/free-slot/lookup processing before invoking policy,
then translates choices to existing Linux results and performs existing staged
publication/erasure. Pure policy decisions do not alter lookup or user-copy
ordering. The predecessor [open-access contract](readonly-open-access.md)
retains its externally visible meaning.

## Native qualification and publication

The library is a third checked object beside root and machine provider. It uses
shared ordinary function lowering, retaining source names, helper calls and
DWARF. Its private entry adapter obeys the target calling convention and guards
invalid representations. The x86-64 implementation disables red-zone and
implicit floating-point use; only integer semantic values enter this profile.
No hosted entry, platform runtime, allocator, foreign dependency, TLS or writable
library data may be linked. Undefined or unexpected symbols fail closed.

LLVM stack-size metadata and the closed acyclic call graph must prove maximum
additional stack use, including wrapper and return addresses, at most 1024
bytes. Dynamic or missing stack evidence is rejected. The initial caller owns
that reserve in the already reserved bootstrap stack; other architectures need
their own calling-convention, frame and caller-reserve evidence. AArch64 and
RISC-V use the same semantic facts/choices, with target-private adapters.

Atomic publication binds source snapshot, semantic interface, selected export,
private Enum adaptation, LLVM tool identity, object digests and stack evidence.
The library snapshot is compiled once into the artifact. No unchecked object
input or numeric symbol identity substitutes for checked source.

Risk is high: purity closure, code ownership, stack bounds, private ABI and
resource-rights enforcement meet at a privileged boundary. Review follows the
ordinary frontend, complete reachable graph, strict profile admission, ordinary
backend reuse, metadata/object inspection, caller reserve/alignment, retained
binding, rights checks, representation conversion and complete terminal erasure.

Qualification shall execute all eight Boolean combinations, inspect actual
source/helper frames in QEMU, retain the existing access/error-order witness and
run the full shared-services corpus. Reordering Enum alternatives must preserve
behavior through adaptation. A changed policy body must change compiled library
behavior while root/provider objects remain independent of it; a deliberately
wrong denial shall be rejected by the Linux behavior witness. Malformed source,
wrong signatures, effects/allocations/foreign calls, cycles, missing inputs and
excess stack must fail before publication. Counterexamples remain separate from
successful kernel qualification.

This is a first source-library qualification, not complete freestanding module,
allocator, arbitrary-type or kernel-subsystem support. See the
[approved architecture](topal-kernel-architecture.md),
[Step 4 status](../step-4-status.md), [gap analysis](topal-gap-analysis.md) and
[baseline](../baseline.md). Core-language meaning is unchanged.

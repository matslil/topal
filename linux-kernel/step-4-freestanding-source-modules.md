# Step 4: ordinary imported source modules

`TK-DEC-257` implements the [closed source-module design](design/freestanding-source-modules.md)
and `TOPAL-SYSTEMS-SOURCE-MODULES-001` for the existing finite and numeric kernel
policy interfaces. The kernel now executes a transitive Boolean module import
and an imported numeric rounding helper. This advances the ordinary-library
obligation in [Step 4](step-4-status.md); general persistent subsystems remain.
Core-language meaning is unchanged.

The primary kernel SHA-256 is:

```text
d391e3f526927567f2e70fbd9c2d0548fe024789194fffcd9093f947a31f1833
```

Artifact/root/provider/object/boot/gate revisions are
`233 / 221 / 222 / 214 / 133 / 233`. Finite and numeric library metadata use
their version-two schemas. Native semantic interfaces and process backing are
unchanged; the complete process backing remains 67 pages, 274432 bytes.

## Implementation and original-design mapping

Filesystem loading resolves the ordinary declared-library/module selection to
a fixed point, freezes exact relative paths, logical identities and source bytes,
and validates the same closure used by explicit in-memory publication. The
initial systems budgets admit 64 selected modules and 1 MiB of root/dependency
source, bound discovery to 4096 entries and 1 MiB of scanned source, and reject
symlinked dependency trees. Duplicate/conflicting/unselected paths or identities,
missing dependencies, undeclared references and cycles fail before publication.
Module runtime initializers and other unqualified declarations are rejected.
Initial imported units contain functions over the selected Boolean/numeric
values. Imported type declarations and cross-unit nominal values remain
unqualified; the finite root owns its Enum adaptation. This prevents incomplete
compiler nominal-module support from exposing private root classifiers.
An independent ordinary checking client instantiates the selected public module
closures without the policy root's private declarations. Thus accidental caller
name capture fails before publication even if later lowering would fold away
the offending expression. A negative case calls a private root function from
an imported helper and is rejected.

The shared frontend checks all reachable functions and preserves ordinary
visibility, signatures, overloads and declared numeric input types. A direct-call
bug discovered by the new tests is repaired: module qualifiers select the
function declaration and do not become operands in the ordinary call resolver.
This conforms to the existing `TOPAL-COMPILER-LIBRARY-SOURCE-001` contract.
The new export-analysis entry point combines existing module and declared-type
selection options; it adds no language rule or filesystem authority.

Both library backends reuse ordinary lowering. All imported calls participate
in the same closed purity/native checks, actual stack metadata and numeric
allocation certificate. Module source ranges select real DWARF files and local
line numbers for functions, variables, lexical blocks and instruction locations.
This source-map installation is selected by the freestanding library emitters.

Publication emits `kernel-library.modules.json` and `time-library.modules.json`
with exact frozen dependency bytes. Library records bind every identity/path/
digest and the snapshot digest; artifact inputs and outputs bind those records.
The boot boundary rejects disagreement between library module records and the
artifact's module-input/snapshot-output provenance. A snapshot grants no machine,
allocator or writable resource authority and never accepts a supplied native
module object.

| Linux responsibility | Current Topal mapping |
| --- | --- |
| Source/header dependencies and subsystem helpers | Ordinary declared source modules with shared visibility/type checking and closed checked calls |
| Build dependency identity | Frozen root/module bytes and per-module identity/path/content digests in atomic artifact provenance |
| Kernel policy versus native mechanism | Boolean facts/choices and arbitrary-precision numeric policy remain ordinary source; native adapters retain checked machine/resource duties |
| Object/debug/resource qualification | One inspected freestanding closure, real imported-source frames, whole-call-path stack proof and complete numeric allocation bound |

This is source-module composition within the artifact. Separate native module
linking, dynamic loading and general Linux loadable-module interfaces are not
qualified. AArch64 and RISC-V preserve the source identity/visibility model while
requiring their own ABI, code placement, frame and runtime-lending evidence.

## Executed evidence

The [finite harness](labs/qemu/x86_64/library_gate.py) observes all eight Boolean
fact combinations, native alignment, callee-saved/SIMD preservation and complete
binding erasure. Its actual backtrace includes `support/logic.t:3`,
`support/boot.t:5`, the root policy and its native bridge. The complete stack
bound remains **180 bytes** within the 1024-byte reserve. Reordered Enum source
still preserves semantic choices; a wrong root denial reaches its separate
specific user-comparison rejection.

The [numeric harness](labs/qemu/x86_64/numeric_gate.py) observes the actual shared
Euclidean runtime called from `support/clock.t:9` and the root deadline policy.
The checked stack bound is **824 bytes** within 4096; the complete conservative
allocation bound remains **41544 bytes** within 65536, with maximum admitted
operand width 93 bits. The shared arbitrary-precision runtime digest remains
`a1c809e3b73bcd168457ca394a0dd4f1613abb68086dc0b97f1e7c168d415de6`.
Actual calls retain native state, bound storage observations and complete
arena/retired-stack/binding erasure.

An additional counterexample changes only the imported rounding helper to return
zero. Its root source digest and native root/provider objects remain unchanged;
the module snapshot and library object change. The existing independent native
deadline oracle observes the wrong scalar before completion. This is a separate
expected rejection, without normal runtime-completion or reclamation claims.
The existing changed-root numeric counterexample is retained independently.

Both harnesses read the emitted snapshot bytes and independently match them to
module records, artifact input digests and the snapshot output digest. The lab
contract checks those exact texts against the repository sources and checks the
real imported frames and changed-helper counterexample. Unit tests exercise
filesystem/in-memory equivalence, transitive calls, visibility, missing/extra/
duplicate/escaping/non-normalized/undeclared/cyclic inputs, initializers,
unbounded imported arithmetic, source/count/entry budgets, symlinks and malformed
provenance bindings. The gate-builder test supplies complete in-memory snapshots.

Validation includes the bounded serial workspace suite, 211 compiler library
tests, 116 semantic-model tests, twenty-two toolchain plus four reference-VM
contracts, formatting, whitespace, relative links and 252 unique systems rules.
After the final module-admission review, focused compiler and gate-builder
validation and both source-library VM harnesses also pass. The retained matrix
binds 56 positive physical QEMU runs to this kernel, plus one positive Enum
variant and seven separate expected rejections.

## Review and remaining acceptance

Risk is high at closed source selection, ordinary visibility, native code
ownership, metadata consistency, numeric resource bounds and debug mapping.
Review traces frozen inputs through the shared frontend and complete source/
runtime graph to object inspection, private adapters, proofs, exact snapshot
publication and terminal return. Independent native arithmetic/byte witnesses,
actual imported frames, changed-source rejection and full previous lifecycle
regression cover different failure modes. No external implementation source or
new external package is imported; the compiler directly uses the existing
workspace syntax crate to reject module initializers before lowering.

The current source interfaces still admit qualified finite/numeric values and
pure acyclic calls. General types, persistent allocation and owned subsystem
state/effects remain. General processes/scheduling/signals, executable/mapping
profiles, writable VFS/devices, networking/isolation, distribution replacement,
Docker/Podman and VM-host acceptance also remain. The initial plan is unfinished.

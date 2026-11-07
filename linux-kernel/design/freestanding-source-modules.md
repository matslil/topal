# Closed source-module snapshots for freestanding policy

`TK-DEC-257`; systems-only direction adopted under standing authorization.
Implemented and qualified for the two pure interfaces; see the
[Step 4 report](../step-4-freestanding-source-modules.md).

Step 2 requires ordinary kernel modules. The two qualified policy interfaces
currently reject imports, which prevents even pure helpers from being shared
between source units. This refinement admits the existing ordinary library
selection, module visibility and direct function calls in those same finite
and numeric profiles. It introduces no new language meaning, interface, native
authority, persistent allocation or unchecked object input.

The first imported units contain ordinary functions over Boolean facts, or the
numeric profile's Boolean/Nat/Int/Euclidean-pair values. Nominal Enum adaptation
remains at the finite root. Imported type declarations and cross-unit nominal
values remain unqualified, so incomplete compiler support cannot expose private
root classifiers to a module or invent a shared nominal identity.

The explicit library root owns discovery. Starting with the selected policy
unit, the publisher resolves ordinary declared libraries and selected qualified
module references to a fixed point using the shared module selection rules.
Each dependency is frozen as a relative source path, logical module identity
and exact source bytes. In-memory publication supplies the same closed snapshot;
it never consults an ambient filesystem. Missing, duplicate, conflicting,
unselected, escaping or cyclic dependencies fail before atomic publication.
Ordinary visibility, overload/type identity and library-version diagnostics
remain the shared frontend's responsibility.

The initial discovery budget is 64 selected dependency units, 1 MiB of total
root/dependency source bytes and 4096 filesystem entries per discovery. The
same byte budget bounds all scanned `.t` files during filesystem discovery,
including files ultimately not selected. Source
paths use contained relative `.t` files; module identities follow the ordinary
file/directory and `module.t` facade mapping. Symlinked dependency trees are
unqualified. These are systems ingestion budgets, not core-language limits.
Publication is over frozen bytes; no atomic snapshot of a concurrently modified
working tree is claimed.

The complete reachable function graph, including cross-module helper calls,
passes the same finite or numeric admission checks and ordinary lowering.
Module runtime initializers, effects, foreign calls, recursion, ambient resource
access and all other previously unqualified forms remain rejected. Module
imports cannot bypass native representation checks or acquire allocator/machine
authority. Numeric export selection retains declared input types rather than
literal client refinements; stack and allocation certificates include imported
helpers and their runtime closure. Debug information retains real module source
locations. The source snapshot and every module identity/path/digest are bound
in provenance, with exact dependency bytes emitted in a digested snapshot output.

Qualified policy units will exercise both direct and transitive helper imports.
Independent qualification must execute actual imported helper frames, preserve
the existing semantic results and native state, observe a changed imported
helper changing behavior while root/provider objects stay independent, and
reject missing/conflicting/unselected/escape/effectful/cyclic dependencies before
publication. Full runtime regression and terminal reclamation still apply.
Cross-architecture differences remain confined to object inspection, private
ABI, code placement, frame bounds and runtime lending; source module identity
and visibility are portable.

Separate native module objects would require a new checked linking and interface
boundary. Flattening source text would lose ordinary module identity/visibility.
A closed ordinary source snapshot reuses the existing checked module machinery
while keeping all code and resource proofs within the current artifact boundary.

Risk is high at source closure, visibility, publication identity and numeric
resource-bound boundaries. Review follows discovery/freeze, exact closure
validation, shared frontend instantiation, every reachable call, unchanged
native/object restrictions and provenance-to-output binding. This closes one
module-support gap; arbitrary persistent subsystem state, general process/VFS/
network services, distribution replacement, containers and VM hosting remain
required by the [Step 4 acceptance ledger](../step-4-status.md).

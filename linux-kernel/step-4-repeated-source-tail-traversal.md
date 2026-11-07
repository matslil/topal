# Step 4: repeated ordinary source traversal

`TK-DEC-282`; follows the
[systems direction](design/repeated-source-tail-traversal.md),
`TOPAL-REQ-SYSTEMS-TAIL-TRAVERSAL-001`, `TOPAL-COMP-SYSTEMS-TAIL-PLAN-001`,
`TOPAL-SYSTEMS-TAIL-PLAN-001` and `TOPAL-SYSTEMS-TAIL-REUSE-001`.

## Stage 1: source structure and progress

The [source-only selector](../src/topal-language/src/compiler_model/kernel_regions.rs)
provides symbolic region/Nat parameter types to the existing ordinary frontend.
For this selection, initial and recursive entry lookup use the same ordered
flattened package signature. Existing checked `Decreases` and Nat adaptation
remain required. The source grammar, operand limit, ordinary package meaning
and existing native qualification paths are unchanged.

The [tail inspector](../src/topal-compiler/src/systems_tail_plan.rs) identifies
the actual natural countdown guard and checks every direct tail edge, including
all argument evaluations and unselected branches. Decrements are positive,
literal and nonnegative under the guard. The optional borrowed region remains
the same input. Helpers must be resolved, reachable and acyclic, without hidden
captures, foreign dependencies, view construction or re-entry. Inferred reads/
writes propagate through helper calls and cannot exceed an explicit empty
effect row. Existing Nat-to-Int widening remains legal; the inspector creates
no narrowing proof.

The result binds exact source/dependency identity/path/bytes and fingerprints
every checked function, including ordered expressions, literals and arguments.
The checked-function fingerprint uses a versioned compiler-internal encoding;
it is no source or native ABI. Its one-step helper-depth fact is not a recursive
stack or logical-identity bound. Descriptive public metadata cannot attest
externally constructed IR or authorize emission.

The actual [clear-prefix source](kernel/algorithms/clear-prefix.t) carries
remaining bytes, offset and success status through ordinary recursion. Helpers
perform byte stores in source order and retain a failed status without rolling
back earlier effects. A separate ordinary read-probe variant tests Optional
loads and transitive read effects. Neither source is selected by the current
kernel or made executable by this source-plan stage.

The [plan tests](../src/topal-compiler/src/systems_tail_plan_tests.rs) exercise
actual shared source/module analysis, packaged recursion and legacy refusal;
independent natural-countdown oracles; all-branch progress; base/non-tail/
argument recursion refusals; guard/step/arity/classifier changes; changed region
transport; helper cycles and missing/duplicate/unreachable functions; source/
checked-graph/dependency binding; hidden/static/resource-result refusal; effect
containment; Nat transport and function/parameter/expression/depth budgets.
Bindings and Optional payloads cannot shadow measure/region storage; distinct
storage with the same display name remains distinct.
IR mutation tests establish defensive inspection only and do not claim source
origin. The eighteen targeted tests and complete affected frontend/compiler
validation pass.

An earlier complete run passed 636 frontend tests and 293 compiler tests before
review added the explicit shadow-storage refusal and two regressions. Its
[historical scope](tools/tail-traversal/results/stage-1-before-shadow-guard/scope.json)
and exact command logs remain separate. That preliminary run bound input hashes
without saving all source bytes and does not qualify the final guard. Fresh final
validation retains exact compressed source snapshots as well as command logs.

The [final record](tools/tail-traversal/results/stage-1/qualification.json)
passes all **636 frontend unit tests** and **295 compiler unit tests** with
no failures or ignored tests; the eighteen tail-plan tests are included in
the compiler total. Locked workspace all-target checking, all four coverage
tests, workspace and included-file formatting, and the unchanged Rust size
ceiling also pass. The systems ledger now contains **312 stable rules**.
Independent readback verifies every source snapshot, original/compressed log
hash and command outcome. The unchanged wrapper used a 2998 MiB adaptive limit,
1569 MiB host reserve and zero cgroup swap, with disk-backed scratch behind a
short owned temporary alias. No host safeguard changed.

The record preserves the verification-input snapshots used by the tests.
Subsequent report/traceability edits record their actual outcomes; they do not
change the qualified compiler/source/normative inputs. This qualification is
source-inspector and compiler regression evidence, not new kernel execution.

Risk is high at termination, recursive signature identity, capability transport,
effects and eventual storage reuse. Review follows ordinary execution, resource,
effect and optimization meaning, then the systems-only requirements/specification
and full diff. It distinguishes structural eligibility from native resource and
ownership evidence. No independent review or new native qualification is claimed.

## Subsequent stages and original acceptance

Stage 2 must prove carried-value ranges and complete live-storage costs,
implement ordinary lowering/state transfer and physically qualify the runtime
checkpoint. The [transfer inventory](knowledge/common/tail-transfer-readiness.md)
records actual current representations and refusal boundaries, with the existing
65536-byte temporary and 4096-byte stack reserves intact.

Stage 3 must use the new execution path on a real owned allocator/metadata
resource, preserving source effects, provenance, reservations, lifetime and
fatal cleanup. Stage 4 must qualify exact committed native/kernel/VM/application
inputs and publish complete evidence in the existing initial Step 4 PR.

General allocation, persistent VFS/process/virtual-memory state, full pinned UAPI
and distribution replacement, Device Tree/drivers, networking and isolation,
Docker/Podman and VM hosting remain required. Other architectures constrain the
abstraction; implementation remains x86-64. The preceding native checkpoint
retains its own [namespace qualification](step-4-shared-namespace-transactions.md).

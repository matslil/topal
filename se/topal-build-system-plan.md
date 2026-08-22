# Topal build-system implementation plan

This plan implements a source-first incremental build and test tool whose
dependency and invalidation policy is ordinary Topal. A narrow Rust host adapter
supplies filesystem observations and process execution until those capabilities
are available directly to Topal. The host must not decide which units are
affected.

## Scope and stable requirements

The initial implementation introduces these requirements:

- **TOPAL-BUILD-GRAPH-001:** every build or test unit shall have one stable
  identity and explicit typed dependencies; unknown identities and dependency
  cycles shall be rejected before execution.
- **TOPAL-BUILD-CHANGE-001:** an exact modification-time mismatch shall mark an
  input changed. Timestamp ordering and final content equality shall not cancel
  that change.
- **TOPAL-BUILD-INVALIDATE-001:** Topal source policy shall select every directly
  changed unit and all reverse-transitive dependents exactly once.
- **TOPAL-BUILD-TEST-001:** tests shall be ordinary units whose dependencies
  declare what they test, so indirect production changes select them through
  the same graph.
- **TOPAL-BUILD-ROOT-001:** source-root-relative inputs and build-root-relative
  outputs/state shall remain distinct. The build root may be inside or outside
  the source tree without changing graph identities or selection.
- **TOPAL-BUILD-BOUNDARY-001:** filesystem observation, atomic state storage,
  directory creation, and process spawning are native capabilities. Graph
  traversal and affected-unit selection are not.

These requirements refine the approved build-system design without introducing
new general language semantics.

## Declarative graph

`topal-build.json` is the initial host-readable projection of the complete
graph. Each unit declares:

- stable `id`;
- `kind` (`build` or `test`);
- source-root-relative `inputs`;
- build-root-relative `outputs`;
- direct dependency unit identities; and
- an argument-vector command executed with explicit source and build roots.

Test units depend on the production or test-support units they verify. The
manifest is rejected if identities are duplicated, references are unknown,
paths escape their root, outputs collide, or the graph is cyclic. Later semantic
frontends may generate this projection at declaration granularity; the graph
contract and Topal invalidation policy remain unchanged.

## Timestamp and state policy

The state database records the complete available modification timestamp for
each declared input. Equality is exact. A missing previous observation, a
missing current input, or any timestamp inequality dirties the owning unit.
No comparison asks which timestamp is newer. A modified-then-reverted file is
therefore changed whenever its timestamp changed.

Successful unit observations are published atomically beneath the build root.
Failed actions never advance state. The state is disposable derived data: its
loss causes a conservative rebuild but cannot alter outputs.

## Root independence

The command accepts `--source-root` and `--build-root`. Relative paths in the
manifest are resolved only against their declared root. The default build root
is `.topal-build` beneath the source root; an absolute or relative explicit
build root supports out-of-tree operation. No output or state path is inferred
from the current working directory.

The same normalized graph identities and affected-unit order must result for
equivalent in-tree and out-of-tree invocations. Integration tests use distinct
temporary roots and verify that out-of-tree execution writes nothing beneath
the source root.

## Native GitHub stack

1. **Plan and requirements.** Establish ownership, graph contract, root model,
   failure behavior, and acceptance evidence.
2. **Topal graph policy.** Add `std build graph` source for membership and
   reverse-transitive invalidation, with executable Topal law tests for direct,
   indirect, independent, duplicate-path, and cyclic-safe traversal cases.
3. **Native host adapter.** Add `topal-build`, strict manifest validation,
   exact timestamp observation, independent roots, atomic state, dry-run
   selection, and process execution that delegates selection to Topal.
4. **In-tree/out-of-tree integration.** Add fixtures for incremental build,
   indirect retesting, test-only changes, failed-action state preservation,
   and root isolation.
5. **Documentation and audit.** Document use and limitations, connect stable
   IDs to Topal and native evidence, run complete validation, and record the
   declaration-level semantic graph frontend as the next admitted increment.

## Acceptance and deliberate limits

The initial tool is complete for explicit manifest units. It does not claim to
extract declaration-level dependencies automatically yet; treating a manifest
input as one unit is conservative. Dynamic observations, remote caches,
distributed execution, filesystem watchers, and platform sandboxes are later
increments.

Every layer passes focused Topal and native tests, formatting, warnings-denied
Clippy, `git diff --check`, and the complete workspace suite under
`systemd-run` with `MemoryMax=4G` and swap disabled. The existing recursive
evaluator tests use an 8 MiB Rust test-thread stack.

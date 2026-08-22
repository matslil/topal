# Topal build tool requirements

These requirements refine the approved build-system design and its source-first
ownership boundary.

## TOPAL-BUILD-GRAPH-001 — Complete explicit graph

The tool shall reject duplicate units, unknown or forward dependencies,
escaping paths, output collisions, empty commands, and unsupported manifest
revisions before executing an action.

## TOPAL-BUILD-CHANGE-001 — Exact timestamp change

The tool shall compare complete available modification timestamps for exact
equality. It shall not use ordering or current content equality to cancel a
detected input change.

## TOPAL-BUILD-INVALIDATE-001 — Source-level selection

Ordinary Topal source shall select changed units and their reverse-transitive
dependents exactly once. The native host shall supply observations and graph
values without implementing a second invalidation algorithm.

## TOPAL-BUILD-TEST-001 — Tests in the dependency graph

Tests shall be graph units whose direct dependencies state what they test.
Changes shall select directly and indirectly affected tests through the same
Topal policy used for build units.

## TOPAL-BUILD-ROOT-001 — Independent roots

All inputs shall resolve beneath an explicit source root. All outputs and
persistent state shall resolve beneath an independently selected build root.
Equivalent in-tree and out-of-tree graphs shall produce the same selection.

## TOPAL-BUILD-BOUNDARY-001 — Narrow native authority

The native adapter may observe files, create build directories, execute
argument-vector commands, verify outputs, and atomically publish state. It shall
not own dependency traversal or affected-unit selection policy.

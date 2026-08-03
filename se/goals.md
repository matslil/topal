# System goals

## TOPAL-GOAL-COMPOSE-001 — One composable model

Language concepts should build from a small set of recursively composable,
first-class objects rather than unrelated special-purpose subsystems.

## TOPAL-GOAL-SAFE-001 — Safety by construction

Safe Topal programs should have defined behavior, explicit failure, checked
bounds and resources, and no data races or internal deadlocks.

## TOPAL-GOAL-DETERMINISTIC-001 — Deterministic meaning

Scheduling and optimization may vary without changing a program's semantic
result.

## TOPAL-GOAL-EXPLICIT-001 — Visible interaction

Effects, dependencies, resource access, protocols, and external boundaries
should be explicit enough for programmers and tools to reason about them.

## TOPAL-GOAL-ZEROCOST-001 — Safe optimization

Pure, immutable source semantics should permit mutation, parallelism, and
specialization when those transformations preserve meaning.

## TOPAL-GOAL-PRECISE-001 — Independent conformance

The language should be specified precisely enough for independent tools to
agree on accepted programs and observable results.

## TOPAL-GOAL-EVOLVE-001 — Traceable evolution

Design changes should remain connected to goals, requirements,
specifications, tests, and implementations throughout language evolution.

# Topal language design

This directory describes how the Topal language works in a form intended to be
read by people designing, implementing, and using the language. These documents
are authoritative for design intent. Exact conformance rules will be maintained
in the repository's formal specifications.

Start with [the design goals](goals.md), then follow the subject documents as
needed. Settled cross-cutting decisions are recorded in
[`decisions.md`](../decisions.md), while deliberately deferred work is recorded
in [`FUTURE.md`](../FUTURE.md).

The design currently covers:

- the object, type, abstraction, capability, and introspection models;
- syntax, execution, errors, effects, functions, and generators;
- modules, constructed contexts, resources, tasks, and interfaces;
- containers, strings, numbers, ranges, units, serialization,
  [data transfers](data-transfers.md), and the
  [incremental build system](build-system.md);
- layouts, addressed storage, sensitive values, tracing, debugger scripting,
  source documentation, generated API reference material, and performance; and
- unit testing, structural path coverage, and the best-practice database.

Revision `v0.2` additionally defines [contracts and evidence](contracts-and-evidence.md),
[synthesized concurrency implementations](concurrency-implementations.md),
[structured transactions](transactions.md), [time](time.md), and
[clocked static-rate dataflow](dataflow.md). These portable semantics do not
introduce source atomics, locks, foreign ABIs, or a hardware architecture
description.

The [design-pattern research library](design-patterns/README.md) is a
non-normative survey used to test the breadth of the core design. Its pattern
requirements and tradeoffs do not state that Topal already supplies a feature.

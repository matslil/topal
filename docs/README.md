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

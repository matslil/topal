# Native serialization requirements

## TOPAL-SERIALIZATION-CANONICAL-001 — Canonical safe streams

The shared native serialization library shall encode and decode protocol 1.0
under `TOPAL-SER-SCOPE-001` through `TOPAL-SER-CANON-001`. It shall emit
canonical bytes, validate revisioned headers and type references before events,
enforce caller-supplied resource limits, consume every frame exactly, and never
reconstruct authority-bearing external resources or capabilities.

Malformed, unsupported, and resource-limit failures shall remain distinct and
shall identify the protocol stage and byte offset. Golden vectors, round trips,
all-prefix truncation, nonminimal encodings, trailing bytes, invalid revisions,
and resource limits shall be covered by deterministic tests.

Understood generic-description values shall be decoded recursively from their
declared schema. The decoder shall not preserve unvalidated remaining frame
bytes as an `ObjectDescription`; component counts, alternatives, and nested
values shall be checked before the event is exposed.

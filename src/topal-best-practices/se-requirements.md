# Best-practice catalog tool requirements

## TOPAL-BEST-PRACTICE-TOOL-001 — Authoritative validation

The tool shall validate every authoritative best-practice record against the
supported schema, identity ownership, lifecycle, classification defaults,
applicability, example references, provenance, and license requirements. It
shall reject unknown schema versions and inconsistent records deterministically.

## TOPAL-BEST-PRACTICE-TOOL-002 — Deterministic projections

`generate` shall write deterministic committed human and agent projections.
`check` shall regenerate in memory and fail when any projection is absent,
stale, or unexpected. Each projection shall identify its source, schema,
generator version, entry version, and authoritative SHA-256 digest.

## TOPAL-BEST-PRACTICE-TOOL-003 — Bootstrap containment

The initial tool may decode the bootstrap JSON encoding but shall preserve the
Topal semantic schema and structured identities. It shall not execute lint
attachments or obtain network authority while validating or generating.

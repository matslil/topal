# Generic export artifact requirements

## TOPAL-GEIR-VALIDATED-001 — Canonical whole-module artifacts

The shared GEIR library shall model the revisioned abstract grammar specified
by `TOPAL-GIR-MODULE-001`, reject a complete module in the validation order of
`TOPAL-GIR-VALID-001` before exposing it as validated, and emit one canonical
byte sequence only from a successfully validated module.

The validator shall preserve exact language revision, stable identities,
types, effects, capability and proof evidence, SSA control flow, visibility,
and the `trusted-unverified` status. Generic substitution shall retain exact
argument and evidence identities and fail when an obligation is absent.

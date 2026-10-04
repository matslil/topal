# Topal kernel design proposals

This directory contains kernel-project design proposals. It is downstream of
the authoritative Topal language intent in `docs/` and system intent in `se/`.
Nothing here changes language meaning by itself.

## Step 2 design set

- [Topal-native kernel architecture](topal-kernel-architecture.md)
- [Topal kernel-readiness gap analysis](topal-gap-analysis.md)
- [Proposed systems profile and elements](kernel-elements.md)
- [Cross-architecture pressure test](architecture-pressure-test.md)
- [Linux-to-Topal mapping](linux-to-topal-mapping.md)
- [Decision register](decision-register.md)
- [Original primitive design criteria](topal-kernel-primitives.md)

The proposal is intentionally semantic rather than syntactic. Approval chooses
the responsibility boundaries and required guarantees; exact syntax is then
designed and propagated through `docs/`, `se/`, `spec/`, tests, and
implementation under the repository change procedure.


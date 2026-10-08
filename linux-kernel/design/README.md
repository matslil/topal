# Topal kernel design proposals

This directory contains the kernel-project design records from which the
approved systems profile was derived. It is downstream of the authoritative
Topal language intent in `docs/` and system intent in `se/`; those authoritative
records, not this directory alone, define language meaning.

## Step 2 design set

- [Topal-native kernel architecture](topal-kernel-architecture.md)
- [Topal kernel-readiness gap analysis](topal-gap-analysis.md)
- [Proposed systems profile and elements](kernel-elements.md)
- [Cross-architecture pressure test](architecture-pressure-test.md)
- [Linux-to-Topal mapping](linux-to-topal-mapping.md)
- [Decision register](decision-register.md)
- [Initial x86-64 Linux boot adapter contract](x86-boot-adapter.md)
- [Bootstrap-region source and provider contract](bootstrap-region.md)
- [Portable boot-memory description contract](boot-memory-description.md)
- [Affine physical-frame allocation contract](physical-frame-allocation.md)
- [Opaque kernel-mapping contract](kernel-mapping.md)
- [Kernel licensing and provenance policy](license-and-provenance.md)
- [Original primitive design criteria](topal-kernel-primitives.md)

The semantic proposal was approved in the project discussion after PR #790.
It is adopted through `docs/systems-profile.md`, the `TOPAL-REQ-SYSTEMS-*`
requirements, and `spec/systems-profile.md`. The approved profile adds no new
grammar; its sealed vocabulary uses ordinary Topal construction syntax.

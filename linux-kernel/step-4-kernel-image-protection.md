# Kernel-image protection: design and portable model

Status: `TK-DEC-285` design/model stage complete under the standing systems-only
authorization. Native construction, enforcement and production qualification
remain unimplemented. The current kernel's mappings are unchanged.

The [design](design/kernel-image-protection.md) preserves the existing
bootstrap-equivalent contract and introduces a separate selected refinement.
The artifact's semantic roles and actual image/root/table owners determine
protection; source does not spell table formats, native addresses or a universal
backing count. Code, immutable data and mutable data have distinct required
rights. Every active and restorable root, including bootstrap return paths,
and every alias of image backing must participate. Existing write protection,
user ownership, mode, publication, exclusion, failure and retirement obligations
remain required. No core-language semantics change.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace this design to
`TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001`,
`TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001`,
`TOPAL-SYSTEMS-KERNEL-IMAGE-PUBLISH-001` and
`TOPAL-SYSTEMS-KERNEL-IMAGE-LIFETIME-001`.
These are requirements for the separately selected refinement,
not a claim that the current provider implements it.

## Portable description checks

[kernel_image_plan.py](tools/user-memory/kernel_image_plan.py) partitions
page-rounded physical image roles. It refuses incompatible roles within one
minimum granule, incomplete relocation initialization, invalid bounds and
geometry. Adjacent compatible ranges merge. Only boundary blocks need individual
subdivision analysis, so large uniform extents remain symbolic rather than
allocating one host object per page. Uncovered ranges mean preserve/inspect
separately, not implicit mapping or permission authority. Split-block counts
are not total hierarchy/backing requirements.

The supplied-view check requires each declared root's primary physical image
coverage and compatible permissions. It rejects writable code/immutable aliases,
executable data aliases, user-accessible kernel aliases, incompatible memory
attributes, holes, contradictory virtual mappings within one root and malformed
descriptive partitions. Additional aliases may narrow rights; code aliases do
not automatically gain execution permission. Virtual and physical addresses
need not be identical.

The [twelve model tests](tools/user-memory/test_kernel_image_plan.py) include
an independent per-cell oracle over all **256** small role/hole patterns, the
current linked x86 layout, 4/16/64 KiB geometries, a symbolic `2^63`-byte uniform
extent, page-rounding conflicts, range overflow, missing bootstrap coverage and
incompatible aliases. Review found a missing same-root virtual-overlap check;
the implementation and a counterexample were added before final qualification.

Under the supplied current x86 roles, preserving the image header needs finer
mapping in its large region as well as the two mixed code/data regions:
**three** split candidates. This is a test of this description, not a hardcoded
native table count. The same current layout is refused for the modeled larger
minimum granules; another provider must choose a compatible placement.

## Executed validation and replay

The [closed archive](tools/user-memory/results/stage-1-kernel-image-plan/report.json)
and [review](tools/user-memory/results/stage-1-kernel-image-plan-review.json)
preserve exact authority/model sources, commands, logs and frozen replay.

- All **271 user-memory** and **82 process-memory** host models pass: **353** total.
- All **762 selected inputs** remain byte-identical across the 3.30-second run.
- No compiler/kernel implementation code differs from the preceding published
  stage. No Cargo build, VM, debugger or generated native code is executed here.
- Model-run peak/limit are **45,015,040 / 3,394,277,376 bytes**, with zero swap
  and all recorded memory-event counters zero.
- Frozen replay restores the exact selected sources and repeats all 353 models.
  Its peak/limit are **83,853,312 / 3,387,641,856 bytes**, with zero swap/events.

The archive has **18 entries**, **753 shared** source snapshots and **9 overlays**.
Its manifest SHA-256 is
`e250d39b6c0cf29acf468a0152f0688f041ac1e60707ff217266318f29711b4d`.
It explicitly depends on the preceding calendar-placement source archive.
Later descriptive status/traceability updates are outside the model run's exact
source-stability claim.

For replay, choose an output directory which does not already exist:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/results/stage-1-kernel-image-plan/methods/replay.py linux-kernel/tools/user-memory/results/stage-1-kernel-image-plan linux-kernel/.cache/x86_64/NEW_IMAGE_PLAN_REPLAY
```

## Scope and next implementation gates

Model objects and successful description checks create no image, root, table or
allocation authority and no protected context. The model checks physical image
coverage, not the native binding of entry/virtual placements. It cannot discover
omitted roots, aliases, CPUs, DMA writers or stale translations; it does not
validate actual mode controls or implement publication/lifetime protocols.
The native provider must establish those facts independently.

Before activating any replacement, review every security invariant and preserve
the obligations of existing exact geometry/ownership checks. Bind actual
compiler/boot placement and backing, update bootstrap and static-process
construction/validation paths together, retain supervisor write protection,
complete architecture maintenance and observe final restoration on the actual
root. The [preceding native audit](tools/user-memory/results/stage-3-calendar-placement.md)
still has writable mappings and 32 unresolved final-root endpoint checks.

Risk is high for the eventual ownership/permission transition. This stage uses
single-agent authority/diff review, independent small-state enumeration,
counterexamples and frozen replay; no independent human security review is
claimed. No native check or activation gate is removed. Full compiler, native
entry/effect/lifetime, cold production, Linux UAPI, DTS, distribution,
Docker/Podman and VM-host acceptance remain unfinished.

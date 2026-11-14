# Whole-root backing descriptors and image requirements

This compiler implementation advances `TK-DEC-285` and `TK-DEC-286` under
`TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001`,
`TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001`. It preserves the existing systems design
and source interfaces. It does not activate an image-protected provider.

## Retained allocation purpose

Native root lowering now retains each allocation's purpose, count and alignment
alongside its actual compiler token slot. Purpose distinguishes bootstrap
hierarchy, process hierarchy, translation-edit storage, payload, low user leaf,
stack and other source extents. Moving or pinning an owner retains that binding.
Generic payload/leaf extents acquire their image purpose only at the consuming
construction transition, which rechecks their complete shape and prior role.
Numeric size alone does not assign that purpose.
The static-process entry snapshot includes the separately retained bootstrap
hierarchy as well as the existing four process/user roles.

The descriptor records each whole extent and its entry pin count, bound to the
exact generated root-object digest and linked token-bank size. Equal numeric
counts do not make bootstrap/process purposes interchangeable. Validation
rejects substituted roles, aliasing slots, changed sizes, alignment or pins.
These are compiler descriptors, not runtime issuer/generation tokens or physical
addresses. Existing runtime whole-owner and USER-table validators remain
unchanged; recording the bootstrap bank does not add it to USER-table admission.

The existing `user-owners.json` observation format is preserved. The additional
`root-backing-owners.json` is descriptive, including the bootstrap binding and
allocation purposes. Serialized descriptors are not accepted as backing
authority.

## Comparison with the actual image

The complete private producer compares the descriptors from fresh root
generation with the exact unmodified root object. Its existing object/link/
load/reservation reconstruction derives the image layout. The new comparison
uses that typed layout directly; it does not deserialize a saved layout or
owner record to construct a resource.

The resulting `hierarchy_backing_requirements` records both banks, the separate
process low-leaf owner, final image identity, raw root-object identity and exact
whole-storage requirements for the draft x86 image-role template. The current
bootstrap-equivalent owners remain five pages each. The current actual image's
draft geometry needs eight pages per bank. This records a storage mismatch;
neither a source integer nor spare pages in another allocation satisfy the
whole-owner obligation. Even an equal size would not prove correct mappings,
runtime ownership, effective controls or publication.

Native selection, runtime-ownership qualification, protected context and
production qualification remain false. Non-image roles, aliases and transition
requirements of the draft geometry remain unqualified. Both producers still
need actual image-bound allocation and construction before protected selection.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-root-backing-owners.json)
records **all twelve commands passing** over **950 stable inputs** in
**239.44 seconds**. Four new owner tests, the original owner-guard test, two
legacy/provider-selected lowering tests, five canonical layout tests, the
default ABI fixture, workspace/all-target checking, formatting/file ceiling,
**271 user-memory** and **83 process-memory** model tests pass.

The ordinary VM case completes the unchanged serial contract through initial
execution and three exec replacements, including **16 valid and 92 refused**
application-buffer requests. All seven static image/descriptor controls are
rejected. The fresh private binding reconstructs **23 objects and 862 allocated
contributions**, checks the exact **336-byte** root token bank, and records
separate bootstrap/process slots **1 and 7**, each five pages with one pin.
The actual image's draft requirement is **eight pages per bank**. Both source
forms produce identical requirements; storage matching remains false.

The raw root object, target LLVM, existing owner-observation JSON and complete
kernel are byte-identical to the preceding `d59dc7b` VM build. The native and
separately rebuilt original-profile kernels retain SHA-256
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
Peak/limit are **1,985,568,768 / 3,410,771,968 bytes**, with zero swap and all
sampled memory events zero. The full local record is
`.cache/x86_64/root-backing-owners-stage-1/run.json`; the actual VM build is
`.cache/x86_64/process-mm-interface-regression-2634134-1794636258031741274`.
This compact result is not a closed source archive or runtime-ownership proof.
The prior 51-workload native corpus is not rerun for this source snapshot.

The reproducible command is:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_hierarchy_backing.py linux-kernel/.cache/x86_64/root-backing-owners-stage-1 --profile root-owners
```

The profile checks new owner-purpose/geometry cases, the original owner guards,
legacy/provider-selected root equivalence, an ordinary serial-only VM case,
complete image reconstruction, canonical layout/default-ABI cases, workspace,
formatting/file ceiling and both memory-model suites. Altered compiler
descriptors cover swapped equal-size roots, aliased bootstrap slots and an
incorrect linked token-bank size. No runtime state is injected. The selected
VM workload uses the existing source-built application and three ordinary exec
replacements. The original-profile image is rebuilt separately.

Review treats ownership and compatibility as high risk. A separate single-agent
diff pass checks purpose capture at allocation and entry, all retained count/
alignment/pin/uniqueness guards, exact root-object association and the boundary
between descriptive requirements and runtime authority. Native instructions
must retain the preceding kernel identity. The historical full compiler suite
and complete Linux/DTS/distribution/container/VM-host acceptance remain open.

An intermediate compiler failure exposed a missing descriptor parameter across
the private guest-builder boundary. The typed parameter was passed through;
no runtime or publication guard was bypassed to obtain a pass.

## Remaining work

The new descriptors and comparisons expose the exact two-root reallocation
obligation. They do not allocate the larger banks, validate runtime ownership
of new geometry, replace existing memory-validation mechanisms or enforce
image permissions. All producers, saved/restorable roots, physical aliases,
effective mode, competing writers, publication and lifetime/fatal behavior
remain required. The full original plan remains unfinished.

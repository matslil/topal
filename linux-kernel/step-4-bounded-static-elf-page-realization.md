# Step 4 bounded static ELF page realization

## Outcome

The compiler now converts the checked bounded static-ELF load plan into an
immutable page-realization plan without reparsing input bytes. The pinned
BusyBox shape yields one future affine 503-frame request with alignment one,
503 ordered virtual-page/frame-ordinal records, a whole-span zero operation,
and four exact segment copies.

Every page begins RW/NX for construction. Final roles contain 396 RO/X pages,
97 RO/NX pages, and ten RW/NX pages. The RO/NX total includes the header,
immutable data, and eight page-rounded RELRO pages. The type model cannot
represent a W+X permission, and exact adjacency prevents undeclared gap pages
from becoming accessible.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PAGE-PLAN-001` adds no core-language element.
The approved systems profile, requirements, formal rule, compiler contract,
and implementation require exact inherited span arithmetic, role order and
adjacency, complete page coverage, sequential ordinals, zero-before-copy
construction, exact copy containment, RELRO containment, and W^X-safe final
permissions.

Focused tests prove the pinned 503-page distribution and construction order.
Negative tests perturb the inherited page count, combined span, adjacency,
role identity, copy extent, and RELRO containment. The complete compiler suite
passes. The sealed two-page image remains the only runtime-selectable input;
the new plan grants no allocation, mapping, copy, permission, or entry
authority and changes no artifact or provider revision.

## Physical regression evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 86
ordered observations. Runtime outputs remain byte-identical to the preceding
gate: linked kernel `a6a1e9273efb8ac2ae915c644600419d21f5f5c173ec87ee57a9c38f73626501`,
boot image `6b39d46dda6c981126e9b171faa2192aa059c1ceeca7e739b814adda196184ee`,
boot resource `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`,
serial output `ac6f1f7eef48dccb7f9214e330bfe7739bbf2aa989a6728360d54711bf16f463`,
and evidence `926ce7f193c6168b30e28121c3fb2a56d2434aaab846777556da58c65e2abf8d`.

## Risk and remaining boundary

Risk is medium: this establishes future mapping and permission facts for an
untrusted executable, but still cannot perform any physical operation. Review
covered inherited-plan distrust, integer and page arithmetic, adjacency,
complete coverage, copy bounds, deterministic zeroing, RELRO, W^X, malformed
cases, runtime nonselection, and unchanged physical execution.

The next increment must extend the bounded affine frame interface and x86-64
provider from its present three-frame maximum to the exact 503-frame
qualification request, without making that bound a portable core-language
constant. Leaf publication, byte construction, final permission transitions,
TLS, stack construction, application entry, and startup syscalls remain later
work.

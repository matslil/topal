# Compiler-derived kernel-image layout

Status: the first compiler component of `TK-DEC-285` is implemented and tested.
Native table construction, complete backing ownership and effective protection
remain unfinished. This component implements part of
`TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001`; it grants no protected context under
`TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001`, `PUBLISH-001` or `LIFETIME-001`.

The opt-in Rust API
[`plan_x86_64_systems_frame_kernel_image`](../src/topal-compiler/src/systems_kernel_image_layout.rs)
rebuilds the root, provider, frame components and every selected policy library
from explicit source snapshots. It requires complete relocated-byte, symbol,
load, allocation and reservation reconstruction before deriving image roles.
An erased library set cannot select standalone component validation. Candidate
metadata must equal the freshly reconstructed record.

Code/immutable/mutable classification comes from those compiler-generated input
contributions. Minimal linker alignment follows their validated output section;
the separately checked read-only ELF header and empty-GOT zero reservation have
explicit immutable roles. Arbitrary ELF flags or a corrected checksum cannot
supply this reconstruction. Nonempty GOT storage remains refused until its
actual generated slots and consumers can be reconstructed.

The provider component rounds roles to 4 KiB pages, refuses incompatible roles
sharing a page and merges compatible adjacent/overlapping ranges. It derives
mixed or partially covered 2 MiB regions from the actual image. Uniform interiors
need no per-page enumeration. The output records the exact final image digest,
role spans, subdivision regions and storage for their additional leaf tables.
Holes remain unspecified. The result is serializable descriptive data, is not
deserializable as a checked resource, and is accepted by no native activation or
ownership interface. These x86 granules remain compiler/provider details.

## Validation

The [five Rust tests](../src/topal-compiler/src/systems_kernel_image_layout_tests.rs)
include all 255 nonempty four-page role/hole patterns, separate empty refusal,
page conflicts, bounds, compatible overlaps and an independent per-page oracle
over every large region of a fresh, fully reconstructed kernel. The complete
source/link test also refuses stale tools, missing library snapshots, altered
placement metadata, changed instruction bytes and an unowned GOT entry. Altered
ELFs are static validator inputs and are never executed.

The [bounded driver](tools/user-memory/run_compiler_image_layout.py) passes all
seven commands: the five Rust cases, default ABI fixture, locked workspace and
all-target checking, formatting, Rust file ceiling, 271 user-memory models and
83 process-memory models. All 764 watched inputs remain unchanged throughout
the 90.54-second successful matrix. Its cgroup peak/limit are
158,572,544 / 3,351,339,008 bytes, with zero swap and recorded memory events.
The [result summary](tools/user-memory/results/stage-2-compiler-image-layout.json)
retains command results, resource observations and the derived layout. Complete
local logs and source hashes are in the driver's output directory; this summary
is not a closed source archive or frozen replay receipt.

The fresh canonical frame artifact is
`4b12d214683e5a3794c4f9af413585a1166d80954e0006fe062eb8106494fb95`.
It derives three image subdivision tables, totaling 12,288 bytes. This is an
artifact-specific additional leaf requirement, not a complete root allocation,
portable table count or evidence for the private process-MM kernel.
No VM, boot-adapter run, debugger injection or execution of generated host code
is part of this matrix. Existing native evidence retains its original scope.

An earlier matrix passed the Rust/static/user checks but failed one saved-suite
model. Its positive fixture mocked historical hashes while discovering today's
compiler file set. The fixture now supplies its historical file set as well.
The changed-byte control isolates one altered historical hash; a new test
requires added and removed compiler files to refuse before hash reuse. The
production qualification/reuse guard is unchanged. The earlier failed run
remains in `linux-kernel/.cache/x86_64/compiler-image-layout-stage-1`; the passing
run is `compiler-image-layout-stage-2`. An initial driver import-path failure
was corrected before those matrices. None is an AI safeguard notice.

To repeat the matrix with a new output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_compiler_image_layout.py linux-kernel/.cache/x86_64/NEW_COMPILER_IMAGE_LAYOUT
```

## Review and remaining work

Risk is moderate for this descriptive compiler component and high for eventual
permission/ownership activation. Single-agent source, authority and complete-diff
review uses independent page-cell checks and reconstruction counterexamples;
no independent human security review is claimed. No current provider, exact
five-page check, mapping builder or publication contract changes.

The private process-MM producer still needs complete reconstruction of its
expanded component roles and GOT storage. Read-only inspection identifies two
current GOT targets: the kernel reservation-end symbol and the frame `brk`
return label. This identifies work, not qualification. Before native activation,
bind the complete topology/backing, both bootstrap and process-root producers,
all restorable roots/aliases, actual controls, publication/maintenance,
competing writers, failure and retirement to the approved
[protection design](design/kernel-image-protection.md). The full compiler suite,
production reconstruction and original Linux/distribution/container/VM-host
acceptance remain unfinished.

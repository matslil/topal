# Step 4 bounded static ELF load plan

## Outcome

The x86-64 compiler now derives one immutable, fail-closed load plan from a
complete bounded static ELF64 input before any of its metadata can influence
artifact construction. The plan records the validated ELF identity, entry,
four ordered R, R/X, R, and R/W load roles, file/memory/page intervals, total
page span, optional TLS and RELRO extents, and non-executable-stack fact. It
contains no borrowed input pointer and grants no allocation, mapping, copy,
permission, or entry authority.

The accepted envelope has at most 16 program headers, exactly four disjoint
load roles, and at most 512 pages wholly within one aligned 2 MiB x86-64 leaf-
table window. The pinned Debian BusyBox shape produces entry `0x4a6580` and a
503-page span from `0x400000` through `0x5f7000`. A synthetic fixture reproduces
that metadata without adding the third-party executable to the repository.

## Validation boundary

Rule `TOPAL-SYSTEMS-STATIC-ELF-PLAN-001` adds no core-language element. The
approved systems profile, requirements, formal rule, compiler contract, and
implementation require complete header/table bounds; exact identity and load
ordering; overflow-free, congruent, disjoint intervals; entry containment;
non-executable stack; contained TLS and RELRO; no interpreter, dynamic loader,
or W+X role; and the 512-page one-window ceiling.

Focused tests derive every pinned shape fact and independently perturb ELF
class, header count, interpreter/dynamic headers, permissions, sizes,
alignment, congruence, file bounds, load ordering, entry, stack, duplicate or
out-of-role TLS/RELRO, span, and required stack presence. The full compiler,
language, and semantics suites pass. Compiler-wide Clippy with warnings denied
remains unavailable as a clean gate because the existing crate and dependency
baseline has unrelated Clippy warnings; the two warnings introduced by this
planner were removed or locally justified.

The existing 12,288-byte sealed qualification image remains the only runtime-
selectable image. A valid plan for any other image returns an explicit
nonselection diagnostic. This increment therefore changes no artifact schema,
provider revision, gate schema, marker set, or generated runtime bytes.

## Physical regression evidence

Two independent pinned `pc-q35-10.2`/`qemu64-v1` QEMU runs reached all 86
ordered observations with identical hashes. The unchanged hashes are:

- linked kernel: `a6a1e9273efb8ac2ae915c644600419d21f5f5c173ec87ee57a9c38f73626501`;
- boot image: `6b39d46dda6c981126e9b171faa2192aa059c1ceeca7e739b814adda196184ee`;
- boot resource: `3353a052b6e9a89540dbad2d2f26494bfb1b8701b3ee967cd4367f16054af008`;
- generated image: `994132429fc84cf9cdbe48042a666418ca4e562f9529b9b62ba6b9f807b627a7`;
- supplied image: `adf24b0776be2281f2b634fa3f1493087e06b489925ce3e8fbd890ddae91b022`;
- artifact provenance: `3ccb97bf81131cc722db89bc31ffc42d9f56574364790fda6529543d6bac44d9`;
- boot provenance: `c13d61c618ca4e5be925d03f363186d5bf266bea763fd2560fb4f38a0825488a`;
- serial output: `ac6f1f7eef48dccb7f9214e330bfe7739bbf2aa989a6728360d54711bf16f463`;
- evidence: `926ce7f193c6168b30e28121c3fb2a56d2434aaab846777556da58c65e2abf8d`.

## Risk and remaining boundary

Risk is medium: the new code parses untrusted executable metadata and performs
security-relevant range and permission classification, but it cannot yet
select, allocate, map, or execute the planned image. Review covered arithmetic
overflow, complete input bounds, page rounding, role ordering, W^X, TLS/RELRO
containment, stack policy, exact pinned facts, malformed cases, and unchanged
physical execution.

The next increment must consume this plan through a bounded affine 503-frame
allocation and segment-mapping lifecycle. Copying file bytes, zeroing BSS,
applying final permissions and RELRO, initializing TLS, constructing BusyBox
arguments/environment, entering the image, and supporting its startup syscalls
remain outside this increment.

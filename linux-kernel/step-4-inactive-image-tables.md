# Inactive kernel-image table data

The compiler test configuration now contains an x86 bank-data planner and a
native buffer initializer for the existing `TK-DEC-285` design. This implements
a preparation component of `TOPAL-SYSTEMS-KERNEL-IMAGE-PLAN-001`. The output
remains descriptive bytes; actual owners, effective protection and publication
under `TOPAL-SYSTEMS-KERNEL-IMAGE-OWNERS-001`,
`TOPAL-SYSTEMS-KERNEL-IMAGE-PUBLISH-001` and
`TOPAL-SYSTEMS-KERNEL-IMAGE-LIFETIME-001` remain unfinished.
The component is private and test-only. It adds no source selection, core
language behavior, production activation or replacement memory validator.

## Geometry and initializer

The [planner](../src/topal-compiler/src/systems_kernel_translation_tables.rs)
describes the x86 root, intermediate directories, reserved high-user table and
necessary image subdivisions. It explicitly supplies a draft supervisor RW/NX
policy outside image ranges, merges equal adjacent roles and subdivides large
regions only where effective rights differ. A partial mutable image inside
mutable coverage therefore needs no redundant leaf table. Code is RX, immutable
data R/NX, and mutable data RW/NX; kernel leaves have no user grant. The bootstrap
and process byte templates retain distinct user-window treatment.

Exact storage follows this declared topology rather than additional image-leaf
bytes alone. Native addresses, four-KiB pages and the fixed prefix remain x86
implementation details. The representational test shape uses eight pages;
this is not a portable count or an allocation request. Numeric bounds and
image/bank/low-table disjointness are checked. A separate USER-table geometry
predicate excludes kernel-only subdivision pages, but establishes no live owner,
entry, permission mode or checked user access.

The [LLVM emitter](../src/topal-compiler/src/systems_kernel_translation_prepare.rs)
produces a closed native function with immutable template/fixup arrays. It checks
numeric geometry, exact length, alignment and the selected kind before stores,
copies the complete buffer, and applies only bounded fixups. Its caller must
already own the exact writable loan. It neither proves that ownership nor reads
physical memory, activates roots, changes CPU controls or returns a protected
context. Final-image binding and all boot/non-image roles still need qualification
before any consumer can use these bytes as live translations.

## Ordinary native execution

The [source-built C program](labs/qemu/x86_64/user_image_table_data_cases.c)
links the emitted object and existing syscall ABI adapter before constructing
its first-party ELF fixture. It uses ordinary `brk` to obtain a heap buffer,
places guards on both sides, compares every output word with a separate
range-based oracle, checks refusals leave the entire loan and guards unchanged,
and restores the previous break. Its physical-number arguments are data inputs,
including hypothetical bounds; they designate no actual owned physical bank.
The selected role partition is a synthetic test description, not authority for
the running kernel image.

An explicitly compile-only helper uses the existing private owning-MM producer,
checks the generated image hash and retains complete 23-object reconstruction.
The legacy public fixture keeps its historical fixed `brk` sequence. A separate
[serial observer](labs/qemu/x86_64/user_image_table_data_check.py) runs the normal
authorized VM, requires the initial program and three successful replacements
to complete their source checks, and verifies the existing console contract.
It records attempted and confirmed execution separately. No debugger injection,
host generated-code execution or pinned third-party binary modification is used.
The kernel follows its existing translation and ownership lifecycle.

## Validation

The [bounded driver](tools/user-memory/run_image_table_data.py) passes all nine
commands in 225.86 seconds over 773 unchanged watched inputs. Its eight selected
Rust tests include seven static cases and the ordinary VM case. Static cases
exercise all 256 small role/hole patterns, independent effective-rights walks,
invalid geometry, bank/image aliases, USER-table role exclusion, LLVM/native
object closure, instruction/stack checks and source-program linking. The native
initializer's source program uses 36,744 text bytes and a conservative combined
stack bound of 1,376 bytes within the fixture's 2,048-byte limit.

Four actual program calls complete 16 valid requests and 92 refusal checks.
All guards and expected words match. The matrix also passes the five canonical
layout cases, unchanged default ABI fixture, workspace/all-target checking,
formatting, Rust file ceiling, 271 user-memory and 83 process-memory models,
and the original private build-only case. Final cgroup peak/limit are
2,538,127,360 / 3,411,296,256 bytes, with zero swap and recorded memory events.
The [result summary](tools/user-memory/results/stage-4-inactive-image-tables.json)
retains commands, source/initializer/image identities, native counts and scope.
Complete local logs and source hashes remain in the driver output directory;
the summary is not a closed source archive or frozen replay receipt.
Descriptive reports were updated afterward; among watched inputs only the
status page changed. The summary records its before/after hashes separately.

Both the running VM and original-profile rebuild produce kernel
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The new first-party application is
`0e21513326aac17d392c304de48da34d7731def33f9e510062f1d3eab28219b5`.
The preceding 51-workload evidence retains its original source snapshot and
scope; this stage does not rerun or requalify that whole corpus. The full
compiler suite, including historical host-RWX and injected-state cases, remains
incomplete.

An earlier isolated authoring copy exhausted its `/tmp` quota and then exposed
omitted standard-library/test inputs. Moving its build into the repository cache
and restoring tracked fixtures corrected those setup failures. Separate static
and native prototype runs then passed before the fresh main-tree matrix above.
Those failed records remain local and are not AI safeguard notices.

## Review and remaining integration

Risk is moderate for this private data-preparation prototype and high for any
future use as live translation storage. Single-agent source/authority and full
diff review uses independent byte oracles, static closure/bounds checks and
ordinary native execution; no independent human security review is claimed.
Existing five-page ownership checks, actual USER callbacks, mapping builders,
permission controls and public qualification gates remain unchanged.

Next preserve the existing purpose-specific hierarchy reservation in checked IR
and derive its complete backing requirement from the qualified final image.
Bind exact issuer/generation/count/alignment/pins and exclusions for both the
implicit bootstrap bank and static-process bank. All writers, aliases, saved
root restorations, effective WP/NXE and publication/maintenance must participate
before activation. The actual final bootstrap hierarchy, complete entries and
asynchronous/CPU/DMA/lifetime obligations remain open. Full Linux UAPI, DTS,
distribution, Docker/Podman and VM-host acceptance remain independent goals.

# User-memory admission research

`LK-X64-USER-MEMORY-001`, `TK-DEC-283`. Scope: facts for
[owned user extents](../../design/owned-user-extents.md), not implementation or
full Linux qualification.

The [exact source record](user-memory-research.json) verifies the selected four
files against the digest-pinned Linux 7.2.9 archive, then against the expanded
cache. Only identities, symbol names and interface facts are retained. No Linux
implementation or generated header is imported.

Linux separates a user-range filter from the accesses which can fault. Its
raw-copy interface reports the remaining byte count, stays within the declared
range and preserves the uncopied suffix. Address tagging depends on the selected
mode. These facts do not make a successful range filter proof of residency or
ownership. Sources: the recorded `arch/x86/include/asm/uaccess_64.h` and
`include/linux/uaccess.h` in the
[pinned tree](https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/?h=v7.2.9).

For ordinary x86 four-level paging, user data access depends on the whole
translation path; write access additionally depends on its write permissions.
Large leaves have distinct entry formats, and optional mode/permission controls
need their own treatment. Source: Intel SDM Volume 3A, order 253668-083US,
March 2024, sections 4.5 and 4.6, available from
[Intel](https://cdrdv2-public.intel.com/819714/253668-sdm-vol-3a.pdf).
This is a specific consulted edition, not a claim to have indexed the newest
manual or every optional CPU feature.

The independent Topal design holds actual resident mapping/table owners through
admission and access in its initial serialized runtime. Linux's internal helper
implementation and page-table layout need not be reproduced. General demand
faults, concurrent mapping edits, signals/restart and syscall-specific partial
results still require their own Linux-compatible contracts and tests. The model
cannot establish hardware stability or grant a capability from numeric facts.

For native `newfstatat`, filename acquisition and lookup precede metadata
conversion and output copying. Lookup failure returns its error without using
the metadata destination. The [exact pinned source record](newfstatat-research.json)
binds these facts to `fs/stat.c` and the x86-64 ABI source. The owned profile
therefore admits pathname input before its loads, and output only when the
query proceeds to publication. This does not generalize the sealed pathname
policies or qualify the complete stat family.

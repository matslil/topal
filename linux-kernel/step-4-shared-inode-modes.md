# Step 4: shared inode mode actions

`TK-DEC-280`; follows the [systems design](design/shared-inode-mode-actions.md)
and `TOPAL-SYSTEMS-INODE-MODE-{STATE,ABI,ACTION}-001`.

## Stage 1: definition and shared-state model

The [portable metadata representation](../src/topal-semantics/src/systems_inode_metadata.rs)
owns the real four-word byte block used by model observations, rather than a
candidate buffer. Existing regular objects, the mutable directory and pipes now
use it for creation, metadata, transfers, truncation, unlink and termination.
Description access rights and positions remain separate from inode permissions.
Pipe creation time remains separate from mutable ctime. Existing model public
observations retain their meanings; pipe observations additionally expose mode
and ctime, and directory metadata has an owned snapshot with no mutation grant.

The admitted prepared-root operation directly updates this shared model state.
It preserves file type and atime/mtime while replacing low 07777 permissions
and recording the supplied mutation-clock observation as ctime. This is a model
of the reference law, not opaque source execution or complete native admission.
The later source action still needs actual-owner loan/exclusion, Boolean
completion and complete reclamation qualification. There is no compiled chmod
handler or new VM claim in this stage.

All **199 semantics tests pass**, including nine new tests:

- All 4096 permission patterns on each of regular/directory/FIFO types preserve
  the exact other bytes, with high native argument bits and full-u64 clock data.
- Repeated equal mode updates ctime, preserves other times and supports complete
  logical metadata erasure.
- Read-only independent and inherited descriptions see the shared mode without
  changing granted rights, contents, allocation, aliases or positions.
- Unlinked old inodes remain distinct from recreated names; closed linked objects
  remain mutable by name, stale descriptions cannot affect reused slots.
- Directory chmod leaves entries/mtime unchanged; later namespace mutations
  preserve the mode and update parent times; termination retires the metadata.
- Wrong owner, same-label foreign pool, invalid name, stale reference and
  poisoned existing controller admissions preserve complete filesystem state.
- Both pipe endpoints and inherited aliases share mode/ctime while preserving
  the complete FIFO, creation time, readiness, counters and endpoint ownership.
- Chmod does not wake a blocked reader; peer close still does. Reused pipe slots
  receive fresh metadata and stale endpoint generations cannot mutate them.
- Equal pipe labels, generations and tickets cannot cross actual pool identity.

The [primary research record](knowledge/x86_64/inode-mode-research.json) binds
seven complete Linux 7.2.9 source files. The actual source archive SHA-256 matches
the pinned manifest, and every researched cached file matches its archive
member byte for byte. The research records syscall numbers, flags, native mode
adaptation, path acquisition and attribute selection. Source research is not
execution against the Linux reference VM; adapter/error-order comparisons remain.

After the user freed memory, the unchanged bounded wrapper admitted the initial
workspace retry with **3072 MiB**, zero swap and **1569 MiB host reserve**; it
passed. The changed-model locked all-target workspace check also passes with
3039 MiB, and the final 199 model tests passed with 3034 MiB. All four
specification/requirement coverage checks pass with 305 unique systems rules.
The first coverage attempt ran before the new traceability row was written and
reported those three missing requirement links; the completed row and full
rerun pass. Formatting, the existing 2000-line Rust ceiling, 478 local links in
changed documents and raw trailing/end/space-before-tab checks pass.
No wrapper limit, host reserve, quota or security policy was relaxed.

Risk is high because shared lifetime, authority, mode bits and native error order
interact. The current review compares all affected model consumers with the
approved design and existing laws, then uses exhaustive independent byte and
complete-state oracles. Pinned Linux execution, independent source/native/GDB
and physical observations are required in subsequent stages. The previous
[TK279 checkpoint](step-4-unsigned-region-fields.md) retains its distinct 2280
Rust tests and 158 VM observations; those counts do not qualify this new action.

## Remaining staged work

Stage 2 must qualify the ordinary two-field source action, actual-owner loans,
strict object/runtime/cost binding and independent guarded native/GDB components.
Stage 3 must integrate all four chmod-family adapters and the real persistent
metadata owners, new exact layout versions, Linux reference controls, physical
cleanup and unmodified BusyBox workloads. Stage 4 must complete the affected
Rust/lab/native/VM cohort and archive exact evidence.

No core language change is needed for this design. General credentials,
filesystem/device services, dynamic programs, distribution replacement,
containers and VM hosting remain part of the original unfinished acceptance.

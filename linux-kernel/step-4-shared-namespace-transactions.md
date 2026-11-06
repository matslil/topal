# Step 4: shared namespace transactions

`TK-DEC-281`; follows the
[systems design](design/shared-namespace-transactions.md) and
`TOPAL-SYSTEMS-NAMESPACE-{OWNERSHIP,TRANSACTION,ABI,ACTION}-001`.

## Stage 1: independent entries and inode ownership

The portable [filesystem owner](../src/topal-semantics/src/systems_volatile_files.rs)
now owns four independent directory-entry slots alongside four regular inodes
and eight resident pages. Entries contain exact raw component bytes and stable
inode identity. Object metadata, page ownership and descriptions stay separate;
link counts come from the actual entry associations. Existing open, stat, chmod,
unlink, directory reads, close and termination use that same table. The owner
rejects dangling identities, duplicate names/inodes, malformed components and
initial-link eligibility inconsistent with a live entry.

The [transaction model](../src/topal-semantics/src/systems_namespace_transactions.rs)
adds hard links by copied name or admitted live descriptor, rename replacement,
NOREPLACE and EXCHANGE. Source and victim inodes retain their identities,
contents, sizes, pages, permissions, atime/mtime and description state. Only
the specified links and ctimes/parent times change. Replacing a victim retains
it through other links or open descriptions; last-reference reclamation erases
its model page associations and bytes. Recreating a spelling issues a fresh
identity. Same-inode rename/exchange preserves the entire state and both names;
NOREPLACE still returns EEXIST.

Initial anonymous temporary files may link through a checked descriptor;
explicit unlinkable creation models O_TMPFILE|O_EXCL. A successful first link
consumes eligibility, so a later open-unlinked inode cannot be resurrected.
Readonly descriptor access is not a link-authority check for the prepared
owning credential and grants no write right. Names, inode numbers, same-label
pools and stale descriptors cannot confer authority.

Entry and inode capacity are independent. A missing-target rename reuses its
entry and can succeed with a full directory. Hard links may exhaust entries
while other inode slots remain free; anonymous files may exhaust inodes while
all entry slots are free. Capacity and other recoverable failures preserve
complete state, including allocation, metadata and identity counters.
Directory cookies now follow entry slots independently of object slots, while
retaining exact 255-byte raw names, 280-byte native record sizes, copy/cursor
ordering and the existing no-snapshot-across-mutation law.

The [primary research record](knowledge/x86_64/namespace-transactions-research.json)
binds six complete Linux 7.2.9 files. The actual archive hash matches the pinned
manifest and each cached source file matches its archive member byte for byte.
Research records all five syscall numbers, flags, same-inode/no-replace order,
temporary-link eligibility, metadata changes and whiteout's separate device
requirement. This is source research, not executed Linux adapter qualification.

The [independent tests](../src/topal-semantics/src/systems_namespace_transaction_tests.rs)
cover shared inode bytes with independent/readonly descriptions, all unlink/
recreate/retained-victim lifetimes, rename without capacity, full-state exact
deltas, exchange/link counts, same-inode no-ops, separate budgets, temporary-link
consumption, raw long names/cookies, foreign/stale/poisoned refusal, malformed
tables, terminal retirement and native unsigned flag adaptation. All **223
semantics tests pass**, including these fourteen new cases, with no failed or
ignored tests. Locked workspace all-target checking and all four requirement/
specification coverage tests pass. The systems ledger has **310 unique rules**;
formatting, the unchanged 2000-line Rust ceiling, local links and strict
whitespace checks pass. The normal bounded wrapper retained its adaptive
memory limit, host reserve and zero swap; no host safeguard changed.

Risk is high because association identity, replacement lifetime, complete
failure preservation, source grants and native argument order interact. Review
compares the actual owner consumers and exact state/observation oracles with
the approved systems design. Direct Rust model transitions and logical
retirement are not opaque Topal execution, physical erasure or syscall evidence.

## Remaining stages and initial acceptance

Stage 2 must implement ordinary namespace source and actual-owner loan/completion
laws, complete source/input/code/data/cost bindings and independent guarded
native/GDB components. Stage 3 must implement all five Linux adapters, V6
dispatch with frozen V1–V5 controls, new versioned entry-table consumers, exact
source-driven effects, Linux/physical lifecycle/refusal cases and unchanged
BusyBox mv/ln. Stage 4 must execute the complete affected committed Rust/lab/
native/VM cohort and publish lossless evidence and accurate scope records.

The selected native kernel is unchanged from
[TK280](step-4-shared-inode-modes.md). Its 2305 Rust tests, 176 VM observations,
45 BusyBox workloads and 43 lab contracts qualify that distinct earlier native
checkpoint. No compiled rename/link handler, new source action or new VM result
is claimed in this first stage. General directories/devices, whiteout, broader
credentials/VFS, allocation, dynamic userspace, distribution replacement,
containers and VM hosting remain part of the unfinished original goal.

# Step 4 increment: reusable static-process file descriptions

Inherited files now support shared reads, seeking, metadata, duplication,
descriptor flags and close through a process-owned table. Pinned BusyBox
`head`, `wc` and `dd` consume the original executable member through those
services. `dd` setup also uses retained native signal registrations. This
advances the x86-64 implementation; distribution replacement, containers and
VM-host qualification remain unfinished.

The systems decisions are `TK-DEC-240` and `TK-DEC-241`. Seven
`TOPAL-SYSTEMS-STATIC-PROCESS-FD-*` rules and
`TOPAL-SYSTEMS-STATIC-PROCESS-SIGNAL-{ACTION,MASK}-001` specify the refinement.
The implementations are `systems_file_descriptions.rs` and
`systems_process_signals.rs` under `src/topal-compiler/src`, integrated with the
existing sealed systems provider. They introduce no core-language change.
Artifact `/219`, root `/208`, provider `/209`, provider object `/201`, boot
adapter `/119` and QEMU gate `/219` identify this contract.

## Ownership and transfers

A separately reserved 4 KiB supervisor page owns sixteen descriptor slots and
sixteen shared descriptions. Slots retain descriptor flags and a description
reference; descriptions retain rights, status, reference count and shared file
position. Lowest eligible slots are selected for allocation. Replacement first
validates its source and target, releases a previous target reference, and
retains the source description. Last close erases that description. The native
`RLIMIT_NOFILE` query reports the actual capacity, sixteen.

The immutable regular file borrows the complete validated `static-user.elf`
member from the reserved boot archive. Provider entry verifies the archive and
table extents remain supervisor identity mappings outside the overridden user
window. Reads use the original member, rather than mutable loaded bytes or a
fabricated header. Each checked transfer stages at most 256 bytes, advances the
shared offset by its result and wipes/readbacks staging. A short read allows
applications to make further progress. Metadata is constructed from the
description, and seek uses checked signed arithmetic with ordinary EOF rules.

Table construction consumes the prelude's former descriptor authority. Failed
exec preserves all descriptors and positions. Successful exec closes only
close-on-exec slots; surviving aliases retain the same shared position. Console
writes resolve live rights through this table, allowing aliases and rejecting
closed or replaced output authority. Serial reads remain a bounded UART service,
not a complete tty.

## Signal setup

The same process page has disjoint storage for 64 native action records and the
blocked set. These resources belong to the process, independently of its files.
Construction imports and consumes the prelude's action facts. Action and mask
operations stage input before optional output, including overlapping buffers.
Size, signal and selector validation follow the pinned native order. Flags and
unmaskable signals are normalized. A valid update commits before an optional
old-value copy; an output `EFAULT` does not undo that update.

Exec retains ignored handlers, resets other handlers and clears every action's
metadata. The blocked set survives. Handler/restorer values remain opaque ABI
data without dereference or application-address matching. Signal delivery,
pending queues, frames, restart and `rt_sigreturn` remain unqualified.

Every terminal status releases all descriptor references, verifies empty slot
and description state, wipes/readbacks the whole process page and clears the
retained table/archive identities before outer reclamation.

## Executed qualification and review

The independently constructed Rust and Python user-mode witnesses exercise:

- alias position sharing, slot/status flag independence, lowest-slot reuse,
  replacement, invalid-source atomicity, table exhaustion and last release;
- complete metadata, rejected/cross-guard destinations, zero/EOF reads, short
  progress, negative/overflowing seeks and valid positions beyond EOF;
- immutable backing after mutation of a corresponding loaded writable byte;
- rejected exec preservation, successful close-on-exec filtering and inherited
  positions, followed by aliased console output;
- signal input/error order, integer-width decoding, overlapping input/output,
  normalization, state after output faults, caught/ignored exec behavior and
  blocked-mask preservation.

The clean fixture emits `runtime-alias` followed by the predecessor output,
returns status `0x23`, and completes rollback while retaining nonempty signal
state until terminal reclamation. Its evidence binds the entire executed
fixture by SHA-256. The lab contract tests verify that binding.

The external corpus retains true/false and two varied echo commands, and adds:

| Command selected through the shell | Exact application output | Status |
| --- | --- | --- |
| `exec /bin/busybox head -c 4 <&3` | bytes `7f 45 4c 46` | 0 |
| `exec 4<&3; exec /bin/busybox head -c 4 <&4` | bytes `7f 45 4c 46` | 0 |
| `exec /bin/busybox wc -c <&3` | `2024544\n` | 0 |
| `exec /bin/busybox dd bs=1 count=4 skip=1 status=none <&3` | bytes `45 4c 46 02` | 0 |

All eight external records in `labs/qemu/x86_64/results/` bind the same linked
kernel and pinned BusyBox. Each checks exact output/status, the complete ordered
kernel markers, continued QEMU liveness and serial quiescence after terminal
disposition. The clean case also uses that linked kernel. The full Cargo
workspace, including all 196 compiler tests, passes through the bounded wrapper.
All seven lab contract tests pass. Formatting validation, whitespace checks,
the 223-rule count and links from the new design/report documents also pass.

Risk is high because aliases, user copying and image lifetime interact. Review
tracks slot/reference-count invariants, native widths/error priority, checked
page permissions, original archive lifetime, staging erasure, exec commit
boundaries and total terminal erasure. Native signal semantics were reviewed
against the pinned `kernel/signal.c`, particularly update-before-output and
clearing ignored-action metadata during exec. File semantics were reviewed
against `fs/file.c`, `fs/fcntl.c` and `fs/read_write.c`. Physical boundary checks
and the independently constructed user fixture supplement implementation review;
human PR review remains required before merge.

Portable ownership stays separate from native descriptor numbers, x86 stat and
signal layouts, register saving, user page tables and return instructions.
AArch64 and RISC-V retain the same slot/description/process-state model with
their own ABI adapters. General path lookup and opens, writable storage,
process creation, dynamic loading, networking, distribution boot, Docker/Podman
and virtualization remain required parts of the initial plan.

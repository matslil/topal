# Reusable prepared-process observations

Status: systems-only direction adopted under standing chat authorization;
implemented and physically qualified (`TK-DEC-247`); see the
[qualification report](../step-4-reusable-process-observations.md).

The prepared single-threaded PID-one process has stable process, thread and
parent observations. Repeated or reordered queries after qualified startup
must observe those retained facts rather than advance a shell-specific call
sequence. The private process owner imports the prepared identity; integer
values and mutable user auxiliary bytes do not grant process authority. In
this profile PID and TID are one and parent PID is zero. Exec retains these
facts and terminal process-table erasure releases their witness. PID namespaces,
process creation, thread groups and mutable identities remain later work.

The prepared system-information view also belongs to the selected artifact,
not an application or output address. Repeated native uname queries expose the
same six 65-byte fields already admitted by the existing profile. The adapter
checks the complete 390-byte writable destination before publishing any byte;
inaccessible/overflowing candidates return EFAULT and preserve process state.
This complete-transfer qualification does not establish partial fault-copy
behavior. Zero padding, all terminators, exact field order and native layout
belong to the ABI adapter. Host system values must not substitute for guest
identity. Broader UTS namespace mutation remains later work.

Queries ignore unused native argument registers, preserve all non-result
registers, retain no caller pointer and do not mutate startup, descriptor,
signal, cwd or image state. They may run across successful exec generations.
The first-image predecessor keeps its existing startup preparation; after the
process table is constructed and startup completes, shared services take over.

Qualification shall execute repeated/reordered PID/TID/parent and uname
queries before and after replacement, complete native bytes and boundary/fault
cases, continued inherited state and terminal rollback. Unmodified BusyBox
shall query system identity and run replacement shells which observe their PID
and execute further commands. Portable meaning remains retained process and
system views; target providers own selectors, widths, records and checked
copies. Core language semantics are unchanged.

Risk is high because kernel-owned identity and user writes are compatibility
and isolation boundaries. Review checks imported authority, observation-only
behavior, complete writable extents, immutable source provenance, native error
and register behavior, exec retention and terminal erasure. The pinned Linux
`kernel/sys.c` PID and uname implementations and existing process/UTS ABI
records are the primary comparison inputs.

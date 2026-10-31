# Reusable static-process file descriptions

Status: approved systems-feature direction under the standing authorization;
implementation and qualification pending (`TK-DEC-240`).

## Purpose and model

Runtime exec, console output and status now work through shared services. The
remaining file operations still recognize individual shell transactions. This
refinement introduces a bounded descriptor table and ordinary file-description
operations, so applications can consume inherited files and shell redirection
can select their input without a per-command provider implementation.

Portable policy distinguishes a process descriptor slot, its descriptor flags,
a shared open description, the description's rights/status flags and position,
and the immutable backing object. A descriptor alias retains the same open
description; it does not copy its position. Closing the last alias destroys
that description exactly once. This is the resource-framework layer of
`TK-KERNEL-LAYERS-001`, adapted to native Linux arguments at the personality
boundary. It does not expose a native file pointer to Topal source.

The first realization has sixteen descriptor slots and a separately owned,
bounded description store. Its configured `RLIMIT_NOFILE` is sixteen, and the
native limit query reports that capacity. Allocation uses the lowest eligible
free slot and reports exhaustion. The finite capacity is a qualification bound,
not an eventual repository-wide process limit or a claim of complete VFS support.

## Construction and executable backing

After the prelude's final serial refill, the process transfers its live
standard descriptions and reopened descriptor three into the reusable table.
Descriptor three retains the existing inode-five immutable executable object,
read-only rights and current position. Standard input and output/error retain
their declared console/serial capabilities. The table is available while the
shell interprets the final runtime command, across checked exec failures, and
after successful replacement. No operation depends on an applet name, payload,
single return address, or a sequence of descriptor numbers from a trace.

The complete executable file is a borrowed immutable view of the already
validated `static-user.elf` boot-resource member. It is not the process's
mutable loaded image, BSS, private pristine-segment copy or a reconstructed ELF
header. The boot-resource reservation remains live until the existing outer
owner releases it. Provider construction checks its bounded address, archive
identity, member extent and disjointness from user-writable storage before
retaining that view. Reads after exec must continue to see original file bytes
even when libc or the application modifies loaded data.

The native realization may add a separate private table-storage argument and
the retained boot handoff to the closed provider call. These remain checked
provider ABI details; they do not add source assembly, general raw pointers,
native register selection, new core effects or a core-language type.

## Shared operations

`read` on an immutable-file description copies bytes from its current position
through private staging, publishes the checked prefix and advances that shared
position by the actual result. A transfer quantum is at most 256 bytes; larger
requests may make short progress and callers can continue until EOF. Bounds
derive from the request, remaining file bytes and transfer capacity. A rejected
nonempty writable candidate must not publish bytes or advance the position.
EOF and zero-byte requests do not read backing bytes or advance the position.
This first refinement does not make the serial-input endpoint a complete tty.

`lseek` on the immutable regular file observes a signed native file offset and
implements set/current/end origins with checked arithmetic. Positions beyond
EOF are valid within the declared signed offset limit and subsequent reads
return EOF. Negative or overflowing resulting positions fail unchanged.
Nonseekable endpoints report `ESPIPE`. Additional native seek selectors require
their own explicit disposition; they must not be inferred from the set/current/
end evidence.

`fstat` publishes a complete native value from the backing object's stable
identity, size, mode, ownership and declared attributes. Descriptor aliases
observe the same inode and attributes. Console/serial descriptions retain their
existing character-device identity. Metadata staging is erased and neither
metadata nor input candidates survive the transfer.

`dup`, `dup2`, `dup3`, `F_DUPFD` and `F_DUPFD_CLOEXEC` create aliases of the
selected open description. Explicit replacement closes the previous target
reference atomically after validating the source and target. `dup2(fd,fd)`
checks liveness and returns that descriptor unchanged; `dup3(fd,fd,flags)` is
invalid. Automatic allocation returns the lowest free slot at or above its
declared lower bound. An invalid source cannot close a valid target.

`F_GETFD` and `F_SETFD` observe/update per-slot close-on-exec state, following
the baseline's native flag masking. `F_GETFL` observes the open description's
access/status flags rather than descriptor flags. An unsupported fcntl command
does not manufacture a success. Closing an invalid or already closed slot
returns `EBADF`; closing a live slot releases one reference and makes that
number available for reuse. A reused number denotes a fresh slot generation.

The console write service resolves the live descriptor table and its rights,
so an alias of a writable console description can publish request-selected
bytes and a closed/replaced slot cannot retain its previous output authority.
Read-only immutable descriptions cannot be written. Existing bounded UART
staging, register preservation and publication ordering continue to apply.

## Exec and terminal integration

Rejected exec requests preserve the table, flags, shared positions and backing
lifetimes unchanged. Successful replacement closes exactly the slots marked
close-on-exec and preserves eligible aliases, rights and shared positions.
It must not recreate a closed standard slot or reset an inherited file offset.
The descriptor table outlives the old image; no pointer into old user memory
is retained by it.

Every terminal status closes every remaining slot, drops each last description
reference once, clears borrowed backing identities and wipes/verifies private
table storage before the existing outer rollback. The predecessor's private
single-description facts are consumed by table construction and cannot remain
an independent source of descriptor authority.

## Qualification and portability

Clean ABI cases must distinguish descriptor flags from shared status/position,
prove lowest-slot allocation, explicit-target replacement and invalid-source
atomicity, exercise close/reuse/last-reference destruction, validate rejected
copies after a nonempty prefix, and read/seek across file boundaries and EOF.
Exec cases must prove unchanged state on failure, close-on-exec filtering and
preserved alias positions on success, followed by complete terminal rollback.
Backing tests must read original file data from a region also loaded writable
and show that changing the process image does not change the file view.

Physical external cases include BusyBox `head` consuming a redirected inherited
file and `wc` consuming the complete executable through the same description
services. Command variations exercise shared offsets and descriptor aliases.
Their bytes and counts must come from application-issued syscalls. Existing
true/false/echo cases remain regressions. Native tracing may locate additional
required calls, but a trace must not become a per-applet acceptance oracle.

Risk is high: this crosses mutable aliasing, resource destruction, image
replacement, user copying and immutable-resource borrowing. Review must track
the reference count against live slots, validate native integer widths and
complete readable/writable extents, check target replacement and error order,
and prove the resource view remains reserved and private through cleanup.
Single-process serialization is the initial synchronization context; concurrent
table mutation and cross-process sharing require explicit later synchronization.

X86-64, AArch64 and RISC-V retain the same slot/description/backing semantics
with their own syscall numbers, native field widths, stat encoding, flags and
copy implementations. The design does not make Linux `struct file` the Topal
resource model. Path lookup, arbitrary new opens, directory enumeration,
writable storage, process creation, dynamic loading, containers and VM-host
qualification remain required later parts of the original plan.

## Baseline evidence

Alternative representations were considered. Per-descriptor copied positions
cannot express dup aliasing. Reusing Linux `struct file` and native pointers
as source types would couple the portable resource model to one implementation.
Adding a separate handler for each application would leave byte transfer and
destruction unshared. The owned slot/description/backing split also supports
later path lookup and filesystem implementations without changing alias rules.

The pinned Linux 7.2.9 source supplies the reference behavior:
`fs/file.c` (`dup`, `dup2`, `ksys_dup3`, `f_dupfd`), `fs/fcntl.c`
(`do_fcntl`, descriptor/status flag distinctions), and `fs/read_write.c`
(`vfs_read`, `generic_file_llseek_size`, `ksys_lseek`). The checked local source
is an ignored input under `linux-kernel/.cache/qemu-x86_64/sources/linux-7.2.9`;
its upstream identity is recorded in [the baseline](../baseline.md).

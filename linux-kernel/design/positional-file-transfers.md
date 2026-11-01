# Positional regular-file transfers

`TK-DEC-268`; systems-only refinement under standing authorization.
Implementation and Topal qualification remain pending.

## Purpose and boundary

Add `pread64` and `pwrite64` over the existing owned sparse files and immutable
boot file, with exact supplied offsets and unchanged open-description positions.
This supplies a reusable storage interface needed by applications and future
VM-host file backing. The [Step 2 architecture](topal-kernel-architecture.md)
and distribution/container/virtualization acceptance scope remain unchanged.
No core-language meaning or source assembly changes.

Select `static-linux-dispatch-v4`, retaining v3's exact 59 operation meanings and
adding `ReadAt`/`WriteAt` (x86-64 selectors 17/18) plus Unsupported. Keep all three
earlier vocabularies and their private exports frozen and explicitly selectable.
All four versions occupy one exclusive dispatch domain with unchanged runtime
and stack reserves. Ordinary source performs operation selection; an adapted
Unsupported choice cannot fall through to a native selector decoder. Finite
file-open policy, object/name budgets and filesystem layout remain unchanged.

## Ownership and native disposition

Positional transfer carries an invocation-owned position rather than borrowing
the shared description's mutable position. It must never temporarily install a
supplied offset in that description, including while clock sampling or user
copying runs. Aliases, inherited descriptions and independent opens retain their
positions after positive, short, zero, EOF and rejected transfers.

The native adapter first rejects a negative signed supplied offset with EINVAL,
before descriptor lookup. A nonnegative offset then requires a live descriptor;
invalid or closed descriptors return EBADF. Console and either pipe endpoint
return ESPIPE before buffer, count or endpoint-direction checks. They do not
block or consume console/FIFO bytes. Seekable regular descriptors retain their
read/write access checks. Pread of a directory follows native numerical range
and signed interval admission before its EISDIR disposition; a read-only
directory descriptor cannot gain write authority through pwrite.

For regular files, preserve the existing numerical user-range admission before
zero/EOF and validate the complete requested signed offset/count interval before
clipping. Use the supplied offset in this check, independent of the shared
position. Then retain the 256-byte/page-boundary transfer policy, complete
returned-prefix admission, private staging, sparse holes, page capacity,
timestamps and failure atomicity of the existing file service. Zero-length and
EOF transfers must not allocate pages, sample a mutation clock, update timestamps
or advance any description.

Pinned Linux's O_APPEND pwrite behavior is part of this profile: after validating
the supplied nonnegative position and requested interval, a nonempty append write
uses current object EOF for placement. It still leaves the description position
unchanged. This is deliberately distinct from assuming that explicit offsets
always override append, and from mutating position to the appended end. No
preadv/pwritev, RWF flags, vector I/O or new append-control flags are added here.

Positional read of the immutable boot file uses its checked borrowed bytes and
never changes the description. Pwrite through its read-only description returns
EBADF. Named and anonymous mutable files, including unlinked objects retained by
open descriptions, use the same sparse-page owner and ordinary object lifetime.
No numeric offset, descriptor or source result becomes storage authority.

## Alternatives and portability

Using seek/read-or-write/restore would expose transient shared state and make
failure, alias and future preemption handling fragile. Duplicating file storage
would break coherent contents and allocation ownership. The implementation
instead parameterizes common bounded transfer realization by invocation-local
versus description-local position, with common copy, page and timestamp rules.
An architecture provider captures the native offset into owned invocation facts;
register names and x86 syscall numbers do not become source-language meaning.
AArch64/RISC-V need their own selector and calling-convention adapters.

Keep the complete native helper bound within the existing 112-byte certificate:
the positional mode must not leave extra saved stack words across the existing
clock path. Reuse the current owned table, page store and staging. Version the
artifact/provider/boot evidence when emitted code and binding selection change;
do not enlarge runtime/stack/storage limits merely to pass qualification.

## Qualification and risk

Risk is high: shared descriptions, signed full-request checks, append, immutable
backing and copy failure interact. Review must follow the supplied position from
the captured syscall through every native helper and confirm no write to a
description's position. The model must resolve sparse pages by object identity,
including independent descriptions whose identity differs from that object.

Use pinned Linux 7.2.9 `fs/read_write.c` (`ksys_pread64`, `ksys_pwrite64`,
`vfs_read`, `vfs_write`, `rw_verify_area`), append realization and an independently
compiled disposable-VM control to establish actual error order and byte/position
laws. Record the complete observed vector before claiming that native adaptation
matches it. Model tests must cover aliases, independent opens, append, empty/EOF,
large sparse offsets, unlinked lifetime, capacity and rejected-copy atomicity.
Independent native source clients must check all four dispatch versions and
reordered nominal choices, exclusive policy combinations and cross-version
rejection. Physical callers must compare actual contents and metadata, observe
unchanged positions, preserve native caller state and verify complete active/
inactive teardown. A deliberately changed result/data case must be rejected by
an independent oracle. Regenerate the existing physical and command corpus for
the new primary configuration and bind current exact artifact identities.

References: [named files](named-volatile-files.md),
[sparse transfers](owned-volatile-files.md),
[descriptions](reusable-static-file-descriptions.md),
[source operation selection](source-linux-syscall-selection.md).

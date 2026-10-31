# Reusable read-only namespace

Status: systems-feature design adopted under the standing chat authorization;
implemented and qualified (`TK-DEC-242`); see the [Step 4 report](../step-4-reusable-readonly-namespace.md).

## Purpose and boundaries

The shared description table supports inherited files. This refinement lets
applications select objects by pathname after exec, using one immutable
namespace and shared lookup, open, metadata, directory and current-directory
services. The initial objects remain root inode three, `bin` inode four and
the artifact-bound `busybox` regular file inode five. Adding objects later must
preserve these semantics rather than add application-specific syscall paths.

Writable storage, links, mounts, concurrent mutation, permissions for multiple
identities and general VFS coverage remain later requirements. The first open
adapter admits read-only requests with native large-file, directory, no-follow,
no-controlling-terminal, nonblocking and close-on-exec modifiers. Other modes
remain explicitly unqualified and fail closed rather than manufacture writable
authority. `openat2`, creation and mutation need separate later adapters.

## Portable model and copying

The process retains a current-directory object identity and namespace root.
Object identities differ from descriptions, native descriptors, names and user
pointers. Each successful open allocates a fresh description at position zero
and a lowest-free slot. Independent opens do not share position; duplication
does. Directory descriptions retain their object and enumeration cookie.
Lookup, metadata, enumeration and reads must agree on inode, type, size, mode
and backing. The regular file uses its complete immutable member size and mode
from the shared-description refinement. Directories have read/search mode;
root has link count three and bin link count two.

Path copying creates private bounded bytes, checking each byte against the live
readable user mapping before access. A terminator must occur within native
`PATH_MAX`. Empty, inaccessible and overlong names remain distinguishable.
Native bytes never become Topal references or ordinary language strings.
Private name storage is disjoint from descriptor, signal, transfer and exec
state and is wiped/read back after every operation and terminal reclamation.

## Resolution and operations

Absolute names start at root and ignore dirfd. Relative names use cwd for
`AT_FDCWD`, or a live directory description. Repeated separators, `.` and `..`
follow directory semantics; root's parent stays root. Regular files cannot be
traversed, including before a trailing slash or dot component. Missing
components return `ENOENT`, invalid relative dirfds `EBADF`, and non-directory
traversal `ENOTDIR`. Rejection preserves process resources. Lookup never
consults host paths, applet identity, a fixed return site or command text.
The initial binding retains the native 255-byte component limit and reports
an overlong component with `ENAMETOOLONG`.

Read-only `open` and `openat` share this model. Native flag width and
normalization precede the native allocation/lookup ordering. A complete name
is copied before allocation; lack of a slot returns `EMFILE` before component
walking. A directory-only request for a regular file returns `ENOTDIR`.
Success atomically installs a fresh description and slot, separating
close-on-exec from shared status flags. Rejection consumes neither resource.
Aliases, flags, close, exec filtering and terminal reclamation retain the
existing lifecycle.

Path and descriptor metadata encode the same object facts. `newfstatat` admits
no-follow/no-automount and empty-path observation with their declared native
error ordering. A complete writable destination is checked before publication;
a missing lookup cannot change it. Broader statx layouts need qualification.

`getdents64` derives complete native records from the live directory and shared
cookie. Root enumerates `.`, `..`, `bin`; bin enumerates `.`, `..`, `busybox`.
Inode, next cookie, record length, type, terminated name and padding agree.
Each call publishes the largest complete record prefix admitted by count and
bounded private staging, advances only for returned records, returns zero at
EOF and rejects a too-small first record without advancing. Regular files
return `ENOTDIR`. Complete checked transfers do not establish Linux partial
user-copy behavior.

`chdir` and `fchdir` commit only resolved directory identities. `getcwd` derives
a terminated absolute path, checks native capacity and complete writable extent
and returns the native length. Failed changes preserve cwd. Cwd survives exec
independently of old user memory and is erased at terminal reclamation.

## Qualification, portability and review

Clean executed cases must cover bad/overflowing/cross-guard/empty/overlong
names, separator/dot/parent/trailing components, absolute names with invalid
dirfds, relative directory/non-directory authority, failure atomicity,
independent opens versus aliases, exhaustion/reuse, close-on-exec, coherent
metadata, record boundaries/shared cookies, cwd failure/exec inheritance and
private erasure. External `head` and `wc` must open by name after exec;
directory-consuming applications must share the same lookup/record services.
Inherited-file and argument/output cases remain regressions.

Risk is high: lookup, copying and object/description publication interact.
Review must check copy-before-allocation, allocation-before-walk, component
boundaries, relative authority, no retained user pointer, native error order
and cleanup. Portable meaning uses owned bytes, object identities, descriptions
and cwd. X86-64, AArch64 and RISC-V supply syscall numbers, flags, layouts,
`PATH_MAX` bindings and copy/return mechanics. No core-language change, source
assembly or native-pointer escape is added.

Primary evidence is pinned Linux `fs/namei.c` (`getname_flags`, `path_init`,
component walking), `fs/open.c` (`build_open_how`, `build_open_flags`,
`do_sys_openat2`), `include/linux/file.h` (`FD_ADD`), `fs/stat.c`
(`vfs_fstatat`, `newfstatat`), `fs/readdir.c` (`getdents64`) and `fs/d_path.c`
(`getcwd`). See [the baseline](../baseline.md).

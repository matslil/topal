# Access intent in the immutable boot-root namespace

Status: implemented and physically qualified under standing systems-only chat
authorization (`TK-DEC-249`); see the [report](../step-4-readonly-open-access.md).

The prepared boot-root namespace is an immutable read-only filesystem view.
Its regular object uses the retained boot-resource member, and it cannot grant
a writable description. Native open flags request access; they do not create
authority or decide whether a pathname exists.

This refinement admits ordinary read-only, write-only and read/write access
intents with the existing qualified modifiers, without creation, truncation,
append, path-only or the nonstandard access-mode-three behavior. Other flag
profiles remain explicitly unqualified. The native adapter normalizes flag
width and copies the complete name, then preserves free-slot-before-walk
ordering. Missing components, invalid relative directory authority and
non-directory traversal retain their ordinary errors regardless of requested
access. Absolute names still ignore dirfd.

After successful lookup, a directory-only request for a regular file returns
ENOTDIR first. Write access to a directory returns EISDIR. Write access to the
existing regular boot-root object returns EROFS under the immutable filesystem
view. Rejection installs no slot/description, changes no position, cwd, signal
or image state and erases copied pathname bytes. Successful read-only requests
retain the existing fresh-description behavior. A future writable filesystem
or device must supply its own object/access authority; this refinement grants
neither. Core-language meaning is unchanged.

Qualification shall execute native open/openat access-intent and error-order
cases, including integer width, bad/overlong names, missing and existing
objects, invalid relative authority, directory-only combinations, descriptor
exhaustion and unchanged inherited state. A fresh unmodified interactive
BusyBox shell shall attempt the absent controlling terminal, then execute
serial prompt/read/command cycles with builtins, cwd changes and nonzero exit
through shared services. Expected prompt text belongs to the harness/caller
environment, never kernel policy. Process creation and job control remain
separate qualification work.

Risk is high because access intent, error ordering and handle publication
interact. Review follows native decoding, copy/allocation/lookup order, object
type before filesystem access denial, no writable authority on failure,
complete pathname erasure, interactive I/O and terminal reclamation. Portable
meaning is access intent under an owned filesystem/object view; x86-64,
AArch64 and RISC-V supply their own flag, selector and error encodings.

Primary evidence is pinned Linux `fs/open.c` (`build_open_flags`,
`do_sys_openat2`), `fs/namei.c` (`open_last_lookups`, `do_open`, `may_open`,
`sb_permission`) and `include/linux/file.h` (`FD_ADD`). In particular, a failed
mount-write attempt does not prematurely replace lookup/type errors. See the
[baseline](../baseline.md) and the predecessor
[read-only namespace](reusable-readonly-namespace.md).

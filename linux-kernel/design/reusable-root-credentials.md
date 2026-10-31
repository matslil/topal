# Reusable sealed-root credential observations

Status: systems design adopted under standing authorization (`TK-DEC-244`);
implementation and qualification pending.

Shared pathname metadata must not depend on advancing a shell-specific
credential sequence. Once the existing image-startup identity preparation has
completed, process queries shall observe the retained real/effective root
user/group identity independently of call order or preceding pathname requests.
The descriptor/process table imports one sealed-root identity witness from the
qualified prelude. Exec retains that process identity; exit erases its witness.
User-writable auxiliary vectors are observations, not identity authority.

Native getuid/getgid/geteuid/getegid return the prepared identity values.
Idempotent native setuid(0)/setgid(0) preserve that same privileged root view and
return zero. Native integer widths apply. Other identity changes remain
unqualified and fail closed; this adds no arbitrary identity/capability or
multi-user policy. Earlier startup preparation keeps its existing adapter;
ordinary later queries have no fixed sequence, pathname, applet or return-site
oracle. Private compatibility observations may record actual credential calls
without becoming their acceptance condition or inventing a metadata event.

Qualification shall cover repeated/reordered queries, idempotent selections,
integer width, exec inheritance, independence from missing metadata, unchanged
descriptions/cwd, native registers and terminal erasure. Risk is high because
identity is authority: review must trace the kernel-owned witness to the
existing prepared root owner and reject user-memory-derived authority.
Portable meaning remains an owned identity view; native IDs and syscalls stay
in the architecture adapter. No core-language change is introduced.

Primary reference is the pinned Linux `kernel/sys.c` identity syscall
implementation; the admitted idempotent root subset does not imply arbitrary
Linux credential changes.

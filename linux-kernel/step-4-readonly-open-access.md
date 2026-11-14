# Step 4 increment: immutable boot-root open access intent

An unmodified replacement BusyBox shell now reaches its interactive prompt,
reads serial command lines, changes cwd and exits with the requested status.
Its ordinary read/write open of the absent controlling terminal observes
ENOENT. Open flags describe requested access; they grant no writable authority
in the prepared immutable boot-root filesystem.

Decision `TK-DEC-249` and rule
`TOPAL-SYSTEMS-STATIC-PROCESS-PATH-OPEN-ACCESS-001` define this increment.
Artifact `/225`, root `/214`, provider `/215`, provider object `/207`, boot
`/125` and gate `/225` identify it. Core-language meaning is unchanged.

## Resolution and error ordering

The native adapter admits access modes read-only, write-only and read/write
with its existing qualified modifiers. It normalizes the native flag width,
copies the complete name, checks a free descriptor slot and resolves components
under the current directory authority. Bad names, missing components, invalid
relative directory authority and non-directory traversal keep their ordinary
errors regardless of access intent. Absolute names ignore dirfd.

After lookup, a directory-only request for the regular object returns ENOTDIR.
Write intent on a directory returns EISDIR; write intent on the regular member
returns EROFS under the prepared immutable filesystem view. Every rejection
erases copied pathname bytes and preserves descriptors, positions, cwd, signal
and image state. No writable description is installed. Read-only success keeps
the existing fresh-description behavior. Creation, truncation, append,
path-only and nonstandard access mode three remain unqualified.

The ordering was reviewed against pinned Linux `fs/open.c`, `fs/namei.c` and
`include/linux/file.h`. In particular, a failed mount-write attempt does not
prematurely replace lookup and type errors. The implementation independently
expresses these observations through the existing owned namespace service.

## Executed qualification

The shared clean semantic corpus, independently encoded in Rust and Python,
checks native open/openat errors for missing and existing objects, normalized
integer width, invalid relative authority, bad/cross-guard pointers,
directory-only combinations, exhausted descriptor slots and unchanged inherited
state. Existing overlong-name and cleanup checks remain in the complete corpus.

Two external interactive cases supply explicit PS1 values through caller-owned
environments. `interactive-prompt` prints `one`, `/` and `/bin`, then exits 7.
`interactive-prompt-changed` prints `two`, changes cwd in both directions and
exits 9. Each shell performs several prompt/read/command cycles after one exec.
Exact application output, including BusyBox's absent-terminal warning and
banner, matches an independent run of the pinned binary under Linux with the
same declared environment. Prompt strings and expected output exist only in
test inputs and observations; the kernel neither recognizes nor generates them.

The matrix has three clean variants, twenty-four external cases and five RTC
cases. All use linked kernel SHA-256
`2380641364b656ce6e0ded5518e764690d6ca6191e1073e021554060603625c3`.
The completed-deadline notification injection also qualifies that kernel; its
predecessor rejection is recorded separately. Full bounded Cargo workspace
tests, fourteen lab contract tests, formatting, whitespace, document links and
the 237-rule inventory pass. Evidence binds actual input/output, status,
replacement count, complete rollback, running QEMU and final serial quiescence.

## Risk review and remaining work

Risk is high because access intent and lookup ordering affect compatibility
and authority publication. Self-review follows flag normalization, complete
copy, free-slot-before-walk ordering, type before filesystem access denial,
unchanged state and erased staging on every failure. Independent fixture
encoders, source review and unmodified shell executions supplement this review;
human PR review and merge remain required.

Portable meaning is access intent under an owned filesystem/object view.
Targets own their flag, selector and error encodings. The
[design](design/readonly-open-access.md) grants no device or mutable filesystem
authority. Fork/wait, shell job control, general executable selection/mappings,
dynamic ELF, writable storage, networking, distribution boot, Docker/Podman
and VM hosting remain required initial-plan work. Interactive builtin evidence
does not establish those capabilities.

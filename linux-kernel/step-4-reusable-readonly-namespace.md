# Step 4 increment: reusable read-only namespace

Static applications can now open the immutable executable by name, resolve
relative paths through cwd or a directory descriptor, enumerate the root/bin
directories and observe coherent metadata. Pinned BusyBox `head`, `wc` and
`ls` use those shared services after exec. Supporting `ls` also adds actual
RTC-backed epoch seconds. Prepared-root credential observations no longer
depend on a particular preceding missing-path request.

This advances Step 4. It does not complete distribution replacement,
Docker/Podman or VM-host qualification. The systems-only decisions are
`TK-DEC-242`, `TK-DEC-243` and `TK-DEC-244`, with eight new formal rules covering
path copying/lookup/open/metadata, directory records, cwd, realtime and root
identity. No core-language change is introduced. Artifact `/220`, root `/209`,
provider `/210`, provider object `/202`, boot `/120` and gate `/220` identify
the qualified implementation.

## Namespace and lifetime

The process table retains cwd and object identities independently of user
memory. Its first page contains descriptor/description state, signal state and
bounded transfer staging. A second private page holds native pathname bytes.
Both pages remain supervisor-only, are checked zero before construction and
are erased/read back before outer reclamation. Every pathname operation also
erases the whole name page before returning.

The walker validates each copied byte and requires a terminator within 4096
bytes. Components retain the 255-byte limit. It resolves repeated separators,
dot and parent components, ignores dirfd for absolute names, enforces live
directory authority for relative names and rejects traversal through regular
files. Root remains inode three, bin inode four and the complete immutable
executable inode five. No host pathname, applet identity, return site or shell
command decides a lookup result.

Read-only opens allocate the lowest available slot and a fresh description
only after complete name copying and successful resolution. Independent opens
have independent positions; aliases retain a shared description. Allocation
exhaustion precedes walking after copying, matching the pinned native order.
Close-on-exec stays separate from shared status flags. Metadata from paths and
descriptors shares one object-value constructor. `getdents64` constructs
complete aligned records with object-derived names/inodes/types and shared
cookies. Rejected transfers leave positions and output unchanged; partial
user-copy behavior remains outside this qualification.

`chdir`, `fchdir` and `getcwd` use retained directory identity. Failed changes
preserve cwd; exec retains it independently of old image memory. General
environment transfer remains later work: the current exec adapter still checks
its predecessor's bounded environment, so syscall-level cwd inheritance does
not imply unrestricted shell environment updates.

## Clock and credential dependencies

Native `time` samples the actual pinned Q35 RTC. It rejects incoherent or invalid
calendar data, supports the qualified BCD/binary 24-hour representations and
converts Gregorian UTC to epoch seconds. A null output queries; a nonnull output
receives exactly the returned seconds after a complete writable check. Private
acquisition/conversion bytes are erased. The backend writes no RTC date/control
data and does not substitute monotonic uptime or a requested test timestamp.

The qualified source range is 1970 through 2261, preceding the pinned RTC
model's signed nanosecond horizon. A wider requested QEMU base can wrap before
the guest observes it; that is not valid evidence for the requested calendar
date. Wider realtime, clock setting, subsecond and synchronization interfaces
remain later work. See the [clock design and primary sources](design/static-process-realtime-seconds.md).

The prepared process imports a private immutable root-identity witness.
Repeated/reordered UID/GID queries and idempotent root selections observe that
witness, not writable auxiliary-vector bytes or a metadata-call sequence.
Exec retains it and terminal table erasure removes it. Other identity changes,
multi-user permission policy and capability changes remain unqualified.

## Executed qualification

The clean user fixture executes the semantic requests in
`labs/qemu/x86_64/namespace-cases.json`. Rust and Python independently encode
those requests into the user image. The evidence binds the entire executed
image by digest, including pre-exec and post-exec witnesses. Cases cover:

- null, overflowing, cross-guard, empty and overlong names; a full-size
  terminated pathname; component/dot/parent/trailing-separator behavior;
- native integer widths, absolute versus relative dirfd authority, failed
  output publication, metadata flag/error order and empty-path observation;
- independent versus aliased file positions, exhaustion/error priority,
  lowest-slot reuse and close-on-exec inheritance;
- coherent root/bin metadata and complete native directory records, shared
  versus independent cookies, record-size boundaries, rejected copies and EOF;
- cwd changes/failures and exec preservation; repeated root observations;
  realtime query/output equality, registers and a rejected partial output.

The standard external corpus retains eight predecessor cases and adds:

| Workload | Exact output | Status |
| --- | --- | --- |
| `head -c 4 /bin/busybox` | bytes `7f 45 4c 46` | 0 |
| `head -c 4 /bin/../bin//./busybox` | bytes `7f 45 4c 46` | 0 |
| `wc -c /bin/busybox` | `2024544 /bin/busybox\n` | 0 |
| `ls -1 /bin` | `busybox\n` | 0 |

Five additional RTC cases run pinned BusyBox `date +%s` against declared virtual
hardware dates: the Unix epoch, the 2000 leap day, the 2100 non-leap century,
the end of 2261 and binary-mode 2000 leap day. The binary case uses explicit
QTest setup while the process waits for its final command: freeze, convert
data, commit representation, resume. Those writes belong to the test setup;
the kernel service stays read-only. Records retain the RTC overrides, encoding,
expected epoch interval, actual application value and complete artifact/serial
provenance. The clock-format transition is itself checked before command entry.

The clean case, twelve standard external cases and five RTC cases all use one
unchanged linked kernel. Each completes ordered rollback markers, leaves QEMU
running and reaches serial quiescence. The full bounded Cargo workspace,
including 196 compiler tests, and nine lab contract tests pass. Formatting,
whitespace, new-document links and the 231-rule count also pass.

## Review and remaining scope

Risk is high because pathname copying, object/description identity, process
lifetime and native clock/identity observations interact. Review follows copy,
allocation, lookup and commit boundaries; component bounds; retained relative
authority; shared positions and last release; complete transfer permissions;
RTC sample/conversion arithmetic; kernel-owned identity authority; and complete
private erasure. Primary Linux sources and independent executed ABI requests
supplement implementation self-review; human PR review is still required.

Architecture-neutral meaning stays owned bytes, immutable objects, descriptions,
cwd and qualified clock/identity views. X86 register layouts, flags, native
records, RTC ports and return mechanics remain in the closed provider.
AArch64/RISC-V require their own adapters. Writable storage, links, mounts,
arbitrary open modes, general environments, broader executable/startup support,
multiple processes, dynamic loading, networking, distribution boot, containers
and VM hosting remain required parts of the initial plan.

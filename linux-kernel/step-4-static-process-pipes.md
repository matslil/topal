# Step 4: shared anonymous pipes and blocked byte transfers

`TK-DEC-256` implements anonymous FIFO objects, inherited endpoints and exact
read/write readiness for the existing static-process pair. The unmodified
BusyBox shell now runs command substitution, including an 8893-byte `seq` output
which exceeds the 4096-byte ring and forces cooperative blocked progress. This
advances [Step 4](step-4-status.md); it does not complete the replacement kernel.

The [design](design/static-process-pipes.md) and rules
`TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-001`,
`TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-TRANSFER-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-PIPE-WAIT-001` define the qualified scope.
Core-language meaning is unchanged. Ordinary Topal subsystem modules remain
required; these services still have a bounded native realization behind the
systems provider.

## Objects, transfers and lifetimes

The first x86-64 binding reserves four FIFO objects, each with a 4096-byte ring.
The supervisor process backing grows from 62 to **67 pages**, **274432 bytes**.
The additional five pages hold shared metadata and four rings. Those pages and
both description banks remain outside private process snapshots. Link/boot
inspection checks the exact complete backing, including rejection of a truncated
predecessor-sized binding with an otherwise updated kernel digest.

`pipe` and `pipe2` publish two lowest-free native descriptors through one checked
eight-byte destination. Each pair owns distinct read/write descriptions and a
fresh FIFO generation. Native flags use their integer width; the qualified stream
profile admits O_CLOEXEC and O_NONBLOCK. Unknown flags are invalid, while valid
packet/notification modes remain explicitly unqualified. Pool and descriptor
exhaustion precede output publication; failed creation returns no partial pair
or leaked endpoint. CLOEXEC belongs to slots, access/nonblocking status to shared
descriptions. Alias/fork/exec filtering and close use the existing reference model.

The shared object retains cursors, count, endpoint presence and coarse timestamps
from the same coherent RTC observation used by native `time`. The clock mechanism
is extracted into a checked internal observation helper; it does not publish a
user result implicitly. FIFO `fstat` reports coherent native type, permissions,
anonymous device/inode identity, size, block size and timestamps. F_GETPIPE_SZ
observes actual capacity; FIONREAD writes an exact checked four-byte count.
Native seek admission resolves the descriptor and unsigned mode before object
dispatch: modes above four return EINVAL, while a pipe returns ESPIPE for admitted
modes independently of offset. This also repairs that precedence for the reusable
console service.

Read/write check direction, numerical range and complete live permissions for
bytes actually transferred before changing payload/cursors. The provider checks
head/count bounds, maintains FIFO order through wrap and erases consumed cells.
Writes through 4096 bytes transfer atomically; larger writes can return actual
positive partial counts. Nonblocking empty/full conditions return EAGAIN, and
empty with no writer returns EOF. The first broken-write qualification returns
EPIPE when SIGPIPE is ignored, including while masked. Nonignored default,
blocked and user-handler signal cases remain ENOSYS until pending/delivery
ownership is implemented; they receive no fabricated signal state.

Blocking transfer retains an exact FIFO-generation condition and native retry,
then selects an eligible counterpart. Readiness follows bytes, sufficient space
for the requested atomic write, or peer closure. A notifier verifies the inactive
request's actual retained slot, endpoint kind and generation before releasing
its condition. Unrelated traffic and ordinary yields cannot manufacture readiness.
Retries re-resolve authority and permissions in the restored process generation;
no checked user pointer, partial copy or live kernel continuation is borrowed
across suspension. A closed-profile deadlock retains its wait obligations.

Last-reference close updates peer presence; both last endpoints erase/read back
the complete ring and metadata before reuse. Parent termination cancels inactive
waits before releasing their endpoint references, then releases active resources.
The outer return verifies and erases all FIFO backing and generation history.

## Mapping from the original design

| Original Linux responsibility | Current semantic design and realization |
| --- | --- |
| Anonymous pipe inode and `pipe_inode_info` | One shared FIFO generation with endpoint identities, cursors/count and timestamps |
| Files and descriptor references | Copied process slots retain shared read/write descriptions; peer presence follows their last references |
| Page/buffer operations | A bounded owned byte ring; no packet, splice or general page-vector contract is inferred |
| Read/write wait queues and sleeping tasks | Exact generation/condition obligations and owned native retries, integrated with the resumable pair |
| Release, wakeup and reclamation | Peer-close readiness, once-only condition resolution, inactive-wait cancellation and checked payload/metadata erasure |

This separates public FIFO behavior from the original C/page-buffer organization.
The first capacity, native layouts and retry instruction are x86 binding details.
AArch64/RISC-V providers require their own page/capacity, ABI, ordering and
return/retry evidence while preserving semantic endpoint and wait ownership.

## Executed evidence

The primary kernel SHA-256 is:

```text
6f5fd985cb20e648211153d33fad69bdd370fc76990549f5832742896eaab3c7
```

Artifact/root/provider/object/boot/gate revisions are
`232 / 221 / 222 / 214 / 132 / 232`.

The independent [native user witness](labs/qemu/x86_64/static_pipe_checks.py)
executes flag/width/error admission, exact pair and FIONREAD publication,
descriptor and FIFO exhaustion, metadata/time bounds, endpoint direction,
aliases/flags, partial reads, atomic insufficient-space rejection, larger partial
writes, wrap, faults, peer EOF, ignored/masked SIGPIPE and generation reuse.
It then forces a child read and child atomic write to block with an inaccessible
candidate: the page fault is checked after readiness, preserving bytes for retry.
Another FIFO's traffic cannot wake the child; half the requested atomic space
does not make its writer eligible. Data, TLS and SIMD state remain private.

The [GDB harness](labs/qemu/x86_64/pipes_gate.py) and
[retained record](labs/qemu/x86_64/results/topal-static-process-pipes-gate.json)
maintain an independent byte queue from actual user inputs. Every positive read
matches its ordered prefix; every ring/count/cursor matches the queue, and unused
cells are zero. The observer checks all description reference counts against
both live slot tables. Nine clean generations are created/reclaimed, with 35
native transfer observations, three blocked calls, six full-state resumptions,
one preserved unrelated condition and two yields with an ineligible counterpart.

The external run observes the unmodified BusyBox shell and child exec. All
**8893 written bytes equal all 8893 read bytes**; the shell removes its trailing
newline and prints the resulting length 8892. Four blocked calls and five
resumptions preserve private backing, permissions, general registers, hardware
FS, XMM/x87/MXCSR state and thread registrations. Shared descriptions and the
entropy history advanced by child exec stay with their owners.

A controlled positive run exits the parent while its child owns a blocked FIFO
read. It confirms cancellation before endpoint destruction, release of all eight
generations reached in that run, and complete backing/registration/library-binding
erasure. Its injected native exit is labeled; it does not claim later application
output ran.

A separate negative run substitutes another live FIFO's generation in a retained
wait. Actual traffic on that unrelated FIFO reaches the native invariant-failure
entry because the saved endpoint still belongs to the original generation.
The debugger verifies both identities and the exact fail-closed instruction
sequence. The observer rejects the input before ordinary completion; timeout
alone is insufficient, and this record claims no normal reclamation.

The complete retained matrix contains **56 positive physical QEMU runs on one
kernel**: three clean environment modes, 34 external workloads, five RTC cases,
three controlled timing cases, completed-deadline injection, finite/numeric
library runs, two preceding child runs, three resumable-pair runs and three FIFO
runs. A reordered-Enum kernel is an additional positive. Six separate expected
rejections cover wrong finite/numeric policies, the historical deadline
predecessor, child/pair snapshot corruption and the wrong FIFO wait generation.

Validation includes the bounded workspace suite, 207 compiler library tests,
116 semantic-model tests, twenty-one toolchain contracts plus four reference-VM
contracts, formatting, whitespace, relative links and 251 unique normative
systems rules. The FIFO model checks ownership, alias/peer lifetime, atomic/wrap/
partial transfers, faults, exact conditions, once-only events, cancellation,
capacity and stale/wrong-pool rejection. A parallel workspace build reached its
1.8 GiB cgroup limit; journal evidence identified OOM termination. The passing
workspace validation uses one build job and one test thread under the same
bounded wrapper/reserve policy, without raising its requested memory limit.

## Review and remaining scope

Risk is high across private/shared separation, endpoint counts, bounded byte
copy, wait registration/readiness, native error order, exec/exit and reclamation.
Self-review traced every publication/ownership boundary, ring extent, reference
transition, retry argument and cancel-before-close path against the authority
chain and pinned native interfaces. Independent model, actual user instructions,
full-state/byte-queue debugger observations, unchanged application execution and
a precise wrong-generation counterexample cover distinct risks. No third-party
source is imported; qualification binaries remain outside version control.

The current profile still has one additional process, one static mapping/image
profile and four fixed-capacity streams. General process tables, preemptive user
scheduling, broader signals/control operations, poll/epoll/splice, named FIFOs,
general/dynamic ELF, writable VFS/devices, networking/isolation, general ordinary
Topal kernel libraries, reference-distribution replacement, Docker/Podman and
VM-host acceptance remain. Full Linux compatibility and the initial plan are
not complete.

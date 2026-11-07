# Step 4 increment: reusable bounded static-process execution

The shell's final command now comes from serial input. The same kernel runs
the pinned BusyBox `true`, `false`, and `echo` applets through shared exec,
console-write, and terminal services. Later arguments and output bytes come
from user requests. The provider checks executable authority and bounds; it
does not select an applet or compare its argument/output payload with a test
string. This advances Step 4; distribution boot remains unfinished.

Decision `TK-DEC-239` and the five `TOPAL-SYSTEMS-STATIC-PROCESS-*` rules link
the systems design, formal contract, implementation, and qualification. The
implementation is in `src/topal-compiler/src/systems_process_runtime.rs`, with
integration in `systems_provider_object.rs`. The separately constructed Rust
and Python fixtures exercise the ABI without implementing an application.
Artifact `/218`, root object `/207`, provider `/208`, provider object `/200`,
boot adapter `/118`, and QEMU gate `/218` identify this increment.

## Transfer and lifetime boundaries

Exec checks one through eight arguments occupying at most 256 bytes including
terminators. Every native vector qword and string byte passes overflow,
interval, and live user-leaf checks before access. A ninth nonnull argument
entry returns `E2BIG` without reading that entry's string. The existing four
environment entries remain checked, and `/bin/busybox` remains the sole sealed
executable. Path, vector, argument and environment failures return Linux errors
before changing the old image. Private staging is wiped and read back on every
failure. Successful transfer alone reaches the indivisible image reset.

The private snapshot's previously unused `[0x9800,0x9a00)` interval holds the
argument bytes, offsets, count, size and copied environment bytes. It is disjoint
from pristine writable ELF bytes ending at `0x9728` and the old upper-stack copy
starting at `0xa000`.
The provider builds a fresh, 16-byte-aligned initial stack at `0x7ffe40`, derives
argv pointers from copied offsets, and constructs environment and auxiliary
vectors including a complete `AT_NULL` pair. Credential readback uses this
generation's retained auxiliary-vector identity. Staging is consumed and
verified zero before entry. The predecessor's image, heap, random, signal,
register, descriptor-inheritance and rollback checks remain in force.

After startup, `write` validates descriptor, count and complete readable extent
in that order. It stages the full zero-through-256-byte request before any UART
output. Both standard output and standard error use this service. Native
`exit` and `exit_group` share cleanup for every low-eight-bit process status.
`TOPAL_KERNEL_PROCESS_EXIT_STATUS=hh` is a kernel-owned hexadecimal status
witness derived from the actual exit argument, after application output.

The final refill copies exactly the received serial prefix, stopping at a
newline or `min(requested_count,256)`. It does not fabricate or compare a command.
The clean fixture requests 23 bytes; the external shell requests 1023. The
earlier four fixed command transactions retain their existing bounds.

## Qualification and review

The clean fixture performs 15 rejected exec requests before successfully reusing
the same image owner. Cases cover null/inaccessible/overflowing candidates,
partial vector extents, an empty vector, unsupported path/argument/environment,
a 256-byte unterminated pathname, aggregate overflow after a copied nonempty
prefix, and a ninth nonnull entry with an inaccessible string. It checks return
errors, a preserved native register and unchanged nonempty source bytes.

The successful clean request contains eight arguments and exactly 256 bytes,
including empty arguments. The replacement checks argc, every argv pointer,
the null terminator and all copied argument bytes. It then checks four write
failures without partial output, a zero-length write with an inaccessible
candidate, a 14-byte stdout write and a 256-byte stderr write. `exit(0x123)`
produces status `0x23` and completes the ordinary resource rollback.

External physical cases are recorded in the lab results directory:

| Workload | Application output | Status |
| --- | --- | --- |
| `true` | empty | 0 |
| `false` | empty | 1 |
| `echo alpha beta` | `alpha beta\n` | 0 |
| `echo omega '' gamma` | `omega  gamma\n` | 0 |

All four records bind the same linked kernel executable and pinned BusyBox
digest. The harness supplies different command bytes to that executable and
checks exact output and observed status. All cases must complete the ordered
kernel markers and remain running and quiescent after terminal disposition.
The six lab contract tests check these evidence relationships. Cargo compiler
and workspace validation run through the repository's bounded wrapper.

Risk is high because runtime native values cross an image-lifetime boundary.
Review covers arithmetic and complete extents before loads, private staging
separation and erasure, the pre-commit failure boundary, value-derived vectors,
native stack alignment, kernel-stack isolation, saved-register restoration,
output atomicity, status masking, and identical cleanup for nonzero status.
Portable meaning remains bounded owned bytes and process/console lifetimes;
pointer width, stack layout, page tables and return instructions remain in the
architecture provider. No core-language design or source assembly is added.

The original plan still requires general executable selection, dynamic loading,
multiple processes, writable storage, networking, distribution applications,
Docker/Podman and VM-host qualification. Earlier shell services and environment
selection remain bounded. This evidence does not establish those later features.

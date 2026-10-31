# Step 4 increment: caller-owned exec environments

Exec now transports bounded caller-owned environments instead of comparing four
fixed strings. Argument zero is independent caller data, and the execution name
has its own auxiliary-vector payload. Startup bytes remain intact during ordinary
stack growth. Pinned BusyBox can export a new variable before exec and observe
its value through `env`, or change cwd and observe it through `pwd`.

This advances Step 4; distribution replacement, Docker/Podman and VM hosting
remain unfinished. Systems decision `TK-DEC-245` introduces no core-language
change. The two formal rules are
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-ENVIRONMENT-001` and
`TOPAL-SYSTEMS-STATIC-PROCESS-EXEC-PAYLOAD-001`. Artifact `/221`, root `/210`,
provider `/211`, provider object `/203`, boot `/121` and gate `/221` identify
this implementation.

## Copy and commit

The adapter admits one through eight arguments occupying at most 256 bytes,
and zero through sixteen environment entries occupying at most 1024 bytes;
both byte bounds include terminators. Null envp and an empty terminated vector
produce no environment entries. Duplicate keys, empty entries and arbitrary
bytes retain their input order without interpretation.

Each vector qword and string byte passes the live readable-mapping check before
access. The seventeenth nonnull pointer returns `E2BIG` without dereferencing
its string. Byte exhaustion also returns `E2BIG`; inaccessible input returns
`EFAULT`. Private staging retains only lengths, offsets and copied bytes.
Rejected requests erase/read back staging and preserve the old image,
descriptions, shared offsets, cwd, signals and prepared identity.

The 2048-byte aggregate occupies snapshot offsets `[0x9800,0xa000)`, after
pristine image bytes ending at `0x9728` and before the rollback-stack copy at
`[0xa000,0xb000)`. It reuses existing private storage. Argument bytes occupy
offsets 0 through 255; their offsets/count/size end at 336. Environment count
and byte size occupy 336 and 344; sixteen offsets start at 352, and environment
bytes occupy `[480,1504)`. Construction and all failure paths wipe the entire
2048 bytes. Successful construction consumes it before user entry.

## Native stack layout

The existing two-page stack remains `[0x7fe000,0x800000)`. Entry RSP is aligned
at `0x7ff800`. At maximum counts, argc/argv/envp and nineteen complete auxiliary
pairs end at `0x7ffa08`, before the first payload:

| Payload | Reserved interval |
| --- | --- |
| Argument strings | `[0x7ffa20,0x7ffb20)` |
| Environment strings | `[0x7ffb20,0x7fff20)` |
| Platform | `[0x7fff20,0x7fff28)` |
| Random bytes | `[0x7fff30,0x7fff40)` |
| Execution name | `[0x7fff40,0x7fff50)` |

Both vector terminators and the full `AT_NULL` pair are zero. All pointers
derive from actual owned offsets; the retained auxiliary identity follows the
actual counts. `AT_EXECFN` names `/bin/busybox` independently of argv[0]. The
whole stack prefix below entry is checked zero before entry. Kernel calls use
the private kernel stack. Portable meaning is isolation from the vector and
stack-growth regions; another architecture supplies its own layout and growth
direction as described in the [design](design/runtime-environments.md).

## Executed qualification

Independent Rust and Python encoders produce the default maximum fixture.
It executes eight arguments/256 bytes, a different argv-zero name, sixteen
environment entries/1024 bytes, repeated keys and empty strings. Additional
executed clean variants cover empty and null envp. Before successful exec,
each fixture checks overflowing/cross-guard vectors, inaccessible strings after
a copied prefix, count overflow with an invalid seventeenth pointer, missing
terminators and aggregate overflow. Failed attempts preserve inherited file
position and close-on-exec state. Post-exec checks compare every vector,
terminator and payload, write 4 KiB below the live stack pointer, verify payloads
again and complete nonzero-status reclamation.

The fifteen external workloads retain the twelve previous cases and add
`pwd-bin`, `export-token` and `export-token-changed`. They observe `/bin\n`,
`TOPAL_TOKEN=alpha` and `TOPAL_TOKEN=omega` through ordinary application output;
the full environment output is checked in the harness. No application name,
environment key or expected output participates in kernel environment policy.
Five RTC date/encoding cases remain regression evidence.

All 23 physical cases use linked kernel SHA-256
`cd1c318075b4599640b78fe78721c34b07240f58406e1c897205ad4645deb5fe`.
They complete ordered rollback, serial quiescence and a still-running QEMU.
The full bounded Cargo workspace passes, including 196 compiler tests, together
with ten lab contract tests. Formatting, whitespace, document links and the
233-rule inventory pass. Evidence resides in `labs/qemu/x86_64/results/` and
binds the actual executed fixture or pinned external image to the artifacts.

## Review and remaining scope

Risk is high because variable untrusted inputs cross destructive image commit.
Review checks byte/count error priority, checked reads, offset bounds, private
staging separation and erasure, stack direction, vector/payload separation,
generation-specific auxiliary identity and preserved state on failure. Existing
formal contracts, independent fixture encoders and executed boundary cases
support self-review; the PR still requires human review and merge.

The executable selection remains the separately admitted immutable object;
general exec-path selection, null/empty argv normalization and repeated exec
remain open. Larger argument/environment limits, dynamic ELF, multiple
processes, writable storage, networking, distribution boot, containers and VM
hosting remain required work. These bounded tests do not establish complete
Linux public-interface compatibility.

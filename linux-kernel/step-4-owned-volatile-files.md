# Step 4: owned sparse volatile regular files

`TK-DEC-265` implements the [owned volatile-file design](design/owned-volatile-files.md)
and `TOPAL-SYSTEMS-VOLATILE-FILE-{OWNERSHIP,TRANSFER,PERSONALITY}-001`.
Applications can create anonymous writable regular files with `O_TMPFILE` in
`/tmp`, read/write actual bytes, seek across holes, truncate/regrow, share
contents and positions through aliases/fork, and retain eligible descriptions
across exec. Last-reference release and active/inactive cancellation reclaim
all storage. [Step 4](step-4-status.md) remains unfinished.

The primary kernel SHA-256 is:

```text
4bf54008151f0c7753c181260c949363da68837a25cf49ec1d5624d6e92639aa
```

Artifact/root/provider/object/boot/gate revisions are
`241 / 227 / 228 / 220 / 141 / 241`. Required artifact evidence now includes
`topal.volatile-file.x86_64/1`: exact owned placement, capacities, private umask,
transfer quantum, signed offset maximum, helper stack bound and timestamp
policy. Numeric metadata stays at version eight. No core-language meaning,
source assembly or hosted kernel allocation is introduced.

## Storage and real native operations

The filesystem has **four** simultaneously live anonymous regular objects and
**eight 4096-byte pages** shared by them. Its separate supervisor-owned region
occupies **36864 bytes**: 4096 bytes of header/object/page metadata and 32768
payload bytes. It follows the existing FIFO region outside private process
snapshots. The complete process backing grows from `0x43000` to `0x4c000`
(**311296 bytes**); linked-symbol validation and physical teardown checks use
the whole new extent. User-writable mappings cannot overlap that backing.

Logical file size and signed positions are independent of physical page capacity.
Holes consume no resident page and return zero bytes. A page has one live object
identity and logical page index. Slots cannot resurrect previous identities or
contents. Allocation exhaustion leaves the file, position and page accounting
unchanged; existing pages remain writable when the shared pool is full.
Truncation erases removed pages and the suffix of a surviving page, so regrowth
cannot reveal removed bytes. Sparse growth and seek allocate nothing.

The native service reuses descriptor lookup, access/status flags, shared
positions, lowest-free installation, dup/fork references, exec filtering,
checked live user mappings and erased transfer/name staging. A transfer returns
at most 256 bytes and ends at its current page boundary. Mode/umask, append,
zero/EOF, metadata, page allocation and error paths operate on the same object;
they do not select application names or retain user pointers. The selected
filesystem uses realtime seconds and noatime, with coherent zero-link metadata
and allocated block counts. `/tmp` has coherent directory records, parent/cwd
behavior and a checked relative-dirfd creation path.

The private umask occupies byte offset **1664**, after transfer staging and
before signal actions. Fork copies it privately and exec retains it; shared
file contents stay outside those snapshots. Compile-time layout checks keep it
disjoint from the existing blocked-signal mask at offset 32. Kernel helper call
depth is bounded by **112 bytes** beyond the unchanged entry: one file-clock
call, twelve saved registers and one RTC call. Its native stack/context guard
retains the existing owned stack range and disabled-interrupt condition. Object,
description and page scans have fixed capacities; inherited RTC sampling keeps
its existing finite retry bound.

## Signed request ranges and versioned source dispatch

`TOPAL-SYSTEMS-LINUX-DISPATCH-V2-001` adds the explicitly selected
`static-linux-dispatch-v2` interface. Its separate
[source unit](kernel/arch/x86_64/policy/static-linux-dispatch-v2.t) extends the
55 existing semantic names with `TruncateFile` (77) and `FileCreationMask` (95).
The original v1 source, exact vocabulary and private export remain independently
supported. The versions occupy one unique dispatch slot and exclusive invocation
domain. Exact source tags, export and native result validation follow the
selected version; cross-version vocabulary/export evidence rejects.

Both native source bridges retain **632-byte** complete invocation stack and
**56-byte** temporary-allocation bounds, including adaptation and saves. Each
version and reversed Enum order executes the independent native client across
65536 selectors, wide inputs and holes. The semantic policy model admits
**47** legal nonempty combinations while rejecting simultaneous dispatch or
identity versions. Adding a handler never redispatches an Unsupported captured
selector natively.

The Linux control exposed a needed correction to full requested-range admission.
After descriptor rights and numerical user-range checks, the complete signed
position/count interval must pass before transfer clipping or EOF. At position
`INT64_MAX-1`, a two-byte request returns `EINVAL`; one byte may reach the maximum,
but another nonzero request fails `EINVAL`. Append checks the original position
first, then selects EOF: an admitted append may make short progress at the maximum
or return `EFBIG` when selected EOF is already maximal. The implementation and
independent tests preserve this distinction.

The same correction applies to the borrowed immutable executable. Its earlier
fixture expected EOF after seeking to `INT64_MAX` and requesting one byte.
That request now returns `EINVAL`, matching pinned `rw_verify_area` ordering.
A successful seek does not admit every subsequent requested count. This updates
`TOPAL-SYSTEMS-STATIC-PROCESS-FD-READ-001` without changing seek or byte-value laws.

## Independent model, Linux control and physical evidence

Seven [sparse ownership/value model tests](../src/topal-semantics/src/systems_volatile_files.rs)
cover actual prefixes, large holes, complete error-state preservation, alias
positions, page/object/reference exhaustion, identity reuse rejection,
truncate/regrow, private umasks, full signed range and append ordering, wrong
owners/pools, stale references and final erasure. Model vectors allocate only
in the host reference model; they supply no kernel storage or native authority.

The [independent C control](labs/qemu/x86_64/volatile_file_control.c) runs on the
pinned Linux **7.2.9** kernel and Debian glibc **2.41-12+deb13u4**, in a disposable
overlay of the qualified reference VM. A noatime tmpfs mount has `size=32k` and
`nr_inodes=5` (its root plus four regular objects). Actual sparse bytes, block
accounting, truncate/regrow, capacities, overwrite during exhaustion, aliases,
append, signed interval limits, private fork umask, exec and close-on-exec pass.
The [control evidence](labs/qemu/x86_64/results/topal-volatile-file-linux-control.json)
binds source, executable, libc requirements, reference kernel and manifest.
The control uses the guest's libc; no C object/runtime is kernel-linked.

The [caller](labs/qemu/x86_64/static_volatile_file_checks.py),
[physical inspector](labs/qemu/x86_64/volatile_file_inspect.py),
[harness](labs/qemu/x86_64/volatile_gate.py) and
[four observations](labs/qemu/x86_64/results/topal-volatile-file-gate.json)
execute actual user requests and observe protected storage after real returns.
Primary and lifetime fixtures provide **130** and **161** syscall-return
observations (**291** total, including both returns of one actual fork).
They cover `/tmp` metadata/enumeration/cwd/relative authority, exact bytes,
sparse offsets above 2^48, maximum signed interval/append behavior, transfer
quantum/page boundaries, readonly/guarded user faults, quotas, reuse, aliases,
exec retention/close-on-exec, shared child writes/positions and private masks.
Whole-prefix qualification does not establish partial copying inside a faulting
prefix.

The inspector checks unique page/object ownership, clean free slots/pages,
zero truncated tails, capacity and full staging/name erasure. Ordinary exit
reclaims files deliberately left live. A controlled child yield then parent
exit separately proves cancellation of active and inactive inherited references,
followed by complete process/filesystem erasure. That controlled observation
is not a successful ordinary userspace run.

A controlled payload corruption changes a real retained page after a successful
write. The caller's independent byte comparison rejects the subsequently read
value at its exact failing comparison; successful scalar return values alone
would miss it. The expected rejection stops before ordinary completion and
does not claim terminal cleanup. The two positive fixtures use identical kernel
bytes despite different supplied caller images.

Boot adaptation rejects **20** altered filesystem profile fields plus missing
profile evidence, including the wrong whole backing extent, page/object
capacities, offsets, signal-mask/umask overlap, quantum, signed maximum,
helper-stack bound and timestamp policy. Existing retention/identity certificate
checks remain intact. These checks supplement actual ownership/byte evidence.

## Regression, review and remaining acceptance

The retained Topal cohort contains **94 VM observations**: **61** successful
primary runs bound to the kernel hash above, **14** successful configuration/
compatibility variants, **six** controlled observations and **13** expected
rejections. The pinned-Linux control is a separate additional VM boot. Controlled
states and expected failures are not counted as ordinary userspace successes.

The complete BusyBox command corpus and caller environments, finite/numeric
libraries, copied ELF headers, current/frozen source dispatch, all identity
versions, clocks, children, pair handoff, pipes and deadline entry regressions
pass. All **30** toolchain and **four** reference-VM contracts pass. The final
bounded serial Cargo workspace passes, including **240** compiler library,
**261** compiler CLI, **139** semantic-model and **610** language tests.
Formatting, whitespace, relative qualification links, **267** unique normative
systems rules and consistent primary-kernel hashes pass.

Risk is high: sparse allocation, native range/error ordering, alias destruction,
private/shared restoration and complete reclamation interact. Single-agent
review checked approved systems intent, pinned source ordering, shared descriptor
lifetime, bounded native loops/stack, strict artifact evidence and independently
implemented model/caller/Linux/physical observations. Human review remains
required through the existing Step 4 PR. The source-vocabulary compatibility
control and older numeric/identity interfaces remain explicit rather than
silently changing their protocols.

This is a bounded Topal RAM filesystem with declared timestamp/transfer policies,
not complete Linux tmpfs-driver equivalence. Native storage mechanisms and Linux
argument adaptation remain trusted provider code; broader ordinary effectful
Topal subsystems still need qualification. Linked-file creation, rename/unlink,
directories beyond the prepared view, writable mappings, persistent storage,
general processes/scheduling/signals, dynamic ELF, networking, isolation,
distribution replacement, Docker/Podman and VM hosting remain acceptance work.
Other targets must qualify their own ABI, placement, page/copy and timestamp
providers while retaining the same portable ownership and value laws.

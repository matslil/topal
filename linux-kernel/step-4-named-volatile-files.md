# Step 4: named files and independent descriptions

`TK-DEC-267` implements the [named-file design](design/named-volatile-files.md).
The kernel can create and reopen regular names in `/tmp`, truncate on open,
unlink files while descriptions retain their contents, and enumerate live names.
The immutable boot executable remains a separate namespace view. Step 4 remains
unfinished: this bounded RAM filesystem does not establish distribution,
container or VM-host compatibility.

## Ownership and publication

A name retains its object independently of open descriptions. Independent opens
start separate positions over shared contents; dup/fork aliases share their
original description. Unlink erases the exact name and drops its link count,
while every independently opened description continues to reference the original
object. Recreating the spelling issues a fresh inode. The last description frees
an unlinked object; closing a linked object preserves its namespace ownership.
Terminal cancellation releases both process banks before destroying closed
linked objects and erasing the complete backing store.

The primary root now selects ordinary `file-open-decision-v1` and
`static-linux-dispatch-v3`. The latter retains v2's 57 operations and adds Unlink
and UnlinkAt. Frozen v1/v2 dispatch and boot-open selections remain distinct
compatibility controls. All dispatch versions use the same exclusive runtime
domain and existing stack/arena reserves. Selection and nominal adaptation use
ordinary Topal source and semantic names, including when declaration order changes.

The native owner checks the selected action against its resolved target and
requested operation. It reserves a clean description and, for creation, clean
object/name capacity before sampling the clock and publishing ownership.
Truncation samples the clock before erasing pages; other descriptions retain their
positions. `O_RDONLY|O_TRUNC` uses filesystem-owner write authority and returns a
read-only description. Errors before publication preserve contents, names,
positions, timestamps, counters and descriptor installations.

Copied path traversal distinguishes prefixes, final components, dot/dot-dot and
trailing slashes. Unlink checks parent write access before final child lookup,
following the pinned Linux control. Invalid unlinkat flags precede copying;
AT_REMOVEDIR returns ENOSYS and remains unqualified directory removal. New
directories, hard links, rename, symlinks, persistence and general mounts remain
subsequent work.

## Bounded layout

Filesystem profile `topal.volatile-file.x86_64/2` retains four object slots,
eight shared 4096-byte pages, a 36864-byte filesystem and 311296-byte process
backing. Four 256-byte name cells begin at filesystem-relative 1536; names hold
one through 255 exact raw nonzero, nonslash bytes. Separate 512-byte directory
staging begins at 3072; payload begins at 4096. Profile validation rejects
missing, mismatched or changed layout evidence before boot publication.

Directory records carry actual object identity, exact bytes and zero padding.
The largest name needs a 280-byte record. Dot/dot-dot and four object-slot cookies
support bounded continuation, shared positions, rewind and current namespace
contents. No-fit and failed user copies preserve the cursor; successful output
is admitted before publishing progress. Anonymous/unlinked objects are omitted.
Creation/unlink update directory modification/change seconds; file truncation
updates its modification/change seconds even when already empty. The selected
clock policy remains realtime seconds with no access-time updates.

The complete native helper bound remains 112 bytes: the file-clock return
address, twelve saved GPRs and RTC return address are the longest path. Nested
last-description/object/page reclamation and terminal scans stay within that
bound. Pending path facts occupy erased private staging, disjoint from umask and
signal state. Clock sampling clears that staging, so creation explicitly retains
its copied-name pointer and length in saved registers before sampling. Names,
directory staging, metadata and pages remain within the existing store.

## Qualification

The regenerated cohort contains **110 Topal VM observations**: **68** successful
primary-kernel runs, **20** successful compatibility/source variants, **seven**
controlled observations and **15** expected rejections. The pinned Linux controls
remain separate. All six new BusyBox workloads and the earlier command, input,
environment, clock, source-policy, process and pipe suites bind current evidence.
The complete bounded serial workspace passes, including **244** compiler library,
**261** compiler CLI, **147** semantic-model and **610** language tests. All **32**
toolchain and **four** reference-VM contracts pass, together with formatting,
whitespace, relative links and **272** unique normative systems rules. The entire
physical driver, including its Cargo/LLVM/GDB/QEMU children, runs inside the
required `scripts/run_bounded.py` resource scope.

An initial parallel language validation was killed by the required cgroup;
serialized bounded validation passes without increasing or bypassing its limit.
The first workspace run exposed stale boot-helper imports in the packager's
in-memory snapshot; removing those unselected imports restores the exact closed
snapshot, and the complete workspace rerun passes. The evidence contracts now
classify the frozen boot-open module run explicitly and bind it to the separate
frozen-profile ELF rather than the primary file-open configuration.

The linked kernel observed by the named-file gate has SHA-256:

```text
93db9ab17bef51f6d847e9343a4f5062ff8d0e7724ac711a89825f5167b2b391
```

Artifact/root/provider/object/boot/gate revisions are
`243 / 229 / 230 / 222 / 143 / 243`. Numeric-library metadata remains version
eight; finite-library metadata remains version two. The selected dispatch source
retains the 632-byte complete stack and 56-byte maximum allocation bounds.
Independent C clients execute all 65536 low selectors, high-bit variants of each
known selector and five additional wide cases for all three versions, then repeat
with reversed Enum orders: 393930 native bridge calls. These qualification clients
are never kernel-linked. All 63 legal nonempty numeric-policy combinations retain
exclusive shared-runtime ownership.

The portable model separates description identity from object identity and uses
the latter for every sparse-page association. Its named-file tests independently
check reopened positions versus aliases, shared data, unlink/recreate identity,
last-description retention, read-only truncation, linked/unlinked capacity,
255-byte raw names, 280-byte directory records and copy/no-fit atomicity.

The [Linux control](labs/qemu/x86_64/named_control.py) runs an independently
compiled caller in a disposable overlay of the pinned Linux 7.2.9 VM. It uses
bounded writable and read-only tmpfs views and asserts nineteen exact error
dispositions alongside the generic ownership and byte laws. It claims no full
tmpfs-driver equivalence.

The [physical caller](labs/qemu/x86_64/static_named_file_checks.py) checks actual
syscall results, contents, metadata and directory bytes. The separate
[inspector](labs/qemu/x86_64/named_file_inspect.py) checks object/name/page and
independent-description associations, failure atomicity, private staging and
complete terminal reclamation. The lifetime fixture covers exec, close-on-exec,
fork, private umask, child writes/unlink and retained independent descriptions.
An inactive-child cancellation case checks both process banks. Deliberate
payload corruption stops at the caller's independent byte comparison.

The primary caller passes **284** actual syscall-return checks and leaves no
file objects. The exec/fork lifetime caller passes **333**, leaving one closed
linked and one open unlinked file with two resident pages for terminal cleanup.
Both complete the ordinary user fixture and erase the entire backing store.
The inactive-child case observes **307** returns before controlled parent exit
and complete active/inactive cleanup. The wrong-data case observes **138**
returns before corruption is rejected at the independent comparison; it claims
neither normal completion nor terminal cleanup.

The existing open-plan source-order variants each pass **83** actual policy
returns with their complete private ABI and binding lifetime; their **130**
caller-return and **161** lifetime checks remain. Their explicit copied source
roots produce distinct whole ELFs, so they are positive variants rather than
primary-kernel qualifications. The wrong create-plan control observes **73**
source returns and follows the actual prepublication refusal. Its debugger now
anchors that comparison to the invocation shim's stack restoration, since later
named-creation guards legitimately reuse the same immediate. Each of five
successful dispatch configurations observes **1108** source-to-native boundaries,
including both frozen versions; the wrong-operation case rejects before entry
to its native handler.

Unmodified pinned BusyBox workloads exercise shell redirection, append,
truncation, independent reopen/read positions, listing and removal through the
same native filesystem services. They use neither host-side output substitution
nor application-specific kernel handlers.

## Review and remaining work

Risk is high because shared file contents, private process restoration, namespace
ownership, raw user bytes and reclamation interact. Review follows model/object
associations, clean resource reservation, copied-path error ordering, clock and
publication boundaries, every helper's stack use, independent description scans,
native ABI encoding, and cleanup of active and inactive owners. Model, native
compiled source clients, pinned Linux and actual VM callers provide separate
oracles; no external reviewer or independent-agent review is claimed.

Broader ordinary effectful Topal subsystems, persistent allocation, general
processes/signals/dynamic ELF, storage/devices, networking, isolation, distribution
replacement, Docker/Podman and VM hosting remain initial-plan acceptance work.
ARM64 implementation remains outside the approved x86-64 focus.

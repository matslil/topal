# Stage 3: complete recorded paging graphs and protected physical routes

`TK-DEC-283`, `TOPAL-SYSTEMS-USER-EXTENT-001`.

The [recursive model](../../alias_snapshot.py) enumerates every present descriptor
in the six recorded owned tables, including supervisor paths and upper canonical
addresses. It checks the known next-table role before following a descriptor,
distinguishes 4-KiB and 2-MiB leaves, rejects unsupported formats and combines
user, write and execute permissions across all levels. Its finite traversal
budget bounds the offline reader; it introduces no kernel mapping limit.

The [saved-data audit](../../audit_mapping_aliases.py) joins **156 boundary
observations**—helper entry, helper return and copy completion for **52 ordinary
stat/positional-write transfers**—to the existing actual source calls, ELF
symbols, load spans and source owner bindings. These associations use **116
distinct snapshots** and produce **six distinct inventories**. Every inventory
contains **572 user pages**, **512 supervisor leaves**, **1084 total leaves**,
**1089 present descriptors** and **six table visits**. All user mappings match
the actual selected live image, stack and heap backing, including pages outside
the particular syscall buffer. Write permission cannot exceed the mapping owner.

For each of the preceding interval observer's **26 protected physical ranges**,
the audit enumerates every intersecting mapping and verifies complete, unique,
writable supervisor identity coverage. There is no other descriptor-based route
to those ranges in the recorded address space at those boundaries. Expected
supervisor identity mappings of user backing remain valid; user-only backing
uniqueness and protected-range inverse routes are distinct checks.

The [second reader](../../readback_mapping_aliases.py) uses direct positions in
the current boot's table arrays rather than recursive traversal. It imports no
recursive, owner or interval model. It independently reconstructs the complete
leaf/descriptor/visit inventories and all protected ranges and inverse routes,
then compares every association. Its [readback](independent.json) agrees exactly.
This second algorithm deliberately qualifies the recorded fixed topology, not
arbitrary future mappings or another architecture.

All **164 model/reader tests** pass. The [twenty offline controls](checks/controls/controls.json)
reject eleven coherently rebound observation changes and nine coherently rebound
report changes. They cover upper-half supervisor/user aliases, foreign and
wrong-level tables, unrelated user pages pointing at kernel/table storage,
missing identity/user mappings, reserved large-leaf bits, unqualified user huge
leaves, false inventory permissions/addresses/entries/levels and missing or wrong
protected routes. Hashes and references are rebound before semantic rejection;
these controls change saved JSON only and execute no altered guest state.

The [first control-run failure](history/first-control-failure/status.json)
compared Python tuple segments directly with JSON array segments. Normalizing
that representation permits the intended semantic controls to run. The exact
failed control source/log remain; the full initial source set is not claimed
retained. This is a reader-harness assertion, with no reported provider denial
and no new native execution. The earlier unit-test setup failure selected the
admitted page rather than an unrelated page and reached the existing admission
check first; the corrected test reaches the intended global large-leaf check.
No exact log is retained for that earlier setup failure.

The [first frozen replay](history/first-replay-failure/status.json) omitted the
recorded QEMU command needed by inherited interval readback. Restoring all
unchanged native observation attachments fixes the replay input set; it does
not run QEMU. The exact failed replay source and reader log remain separate.

The [native-source record](native-source.json) reuses the preceding complete
[interval VM run](../stage-3-owner-intervals/README.md) without executing a new VM.
Its actual two-CPU TCG configuration and host/guest boundaries remain unchanged.
Kernel SHA-256 remains
`47b785aca3794d171d508a3fdbb616057a613016c1783ca9ec95236dbba2199f`.
All **93 artifact references** and **178 preceding inputs** are inherited;
five new offline sources give **183 selected inputs**. No third-party ELF,
initramfs or additional binary is added to Git.

[Frozen replay](replay.json) restores those selected sources and artifacts,
reproduces all thirteen preceding audits, the exact interval/owner/metadata
readbacks, fourteen preceding controls, all 164 tests, and the exact new
mapping/independent/control reports. It executes Python data readers only:

```sh
rtk proxy env RUST_MIN_STACK=16777216 PYTHONDONTWRITEBYTECODE=1 scripts/run_bounded.py -- python3 linux-kernel/tools/user-memory/results/stage-3-mapping-aliases/replay.py linux-kernel/.cache/x86_64/mapping-alias-replay-new
```

Use a new output directory. [Archive validation](validate-archive.py) binds
sources, inherited inputs, observations, reports, controls, failure history and
replay. The [primary facts](primary-facts.json) identify the consulted Intel SDM
edition and relevant entry-format, permission and translation-cache sections.

This closes the unknown descriptor-alias question for these recorded snapshots.
It does **not** exclude stale TLB entries, alternate or changing paging modes,
DMA, competing AP actions or all asynchronous entries throughout a transfer.
The earlier identity-address watches and these boundary snapshots together are
still insufficient to establish continuous physical protection. No memory
protection check is removed or replacement selected. General stable lifetime,
other callers/private-counterpart loans, complete exception/fatal/whole-syscall
effects, unrun native lengths/refusals, all fallback gates and canonical clean
production reconstruction remain required. Core language and executed kernel
behavior are unchanged. Stage 3, initial Step 4 and full Linux distribution,
Docker/Podman and VM-host acceptance remain unfinished.

[Review](review.json) treats memory/lifetime/evidence errors as high risk, with
single-agent higher-authority review and two separately written algorithms.
No independent human security review is claimed.

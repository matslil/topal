# Retained boot coverage in both inactive banks

`TK-DEC-291` adds complete native page walks for the supported retained boot
roles in both constructed image banks. Fixed extents come from the same adapter
source; dynamic extents come from the actual validated boot binding. Core
semantics, source syntax and existing active hierarchy contracts are unchanged.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through the
[design](design/inactive-boot-coverage.md) to
`TOPAL-SYSTEMS-KERNEL-IMAGE-BOOT-COVERAGE-001`, native checks, read-only captures
and independent saved-byte replay.

The native checker follows complete whole-owner and exact-template validation.
It checks every outward-rounded page in seven fixed boot regions, the actual
boot-parameter page, the actual boot-resource interval and both MMIO windows.
All table-pointer reads remain bounded to the exact owning bank. Effective
ancestor/leaf write, supervisor and NX permissions, identity translation,
supported granules and applicable leaf cache-index positions are checked.
Retired handoff storage remains retained and NX after control has transferred
into the executable kernel image. Device mapping reachability grants no device
access or DMA authority.

The private 64-word journal retains eleven exact extents, their cache indices
and page counts. It is empty before construction, records one completed check
after both banks pass, and records two after complete repeat checking before
discard. An independent four-level interpreter derives those facts from the
actual adapter contract, retained boot inputs and complete captured bank bytes.
It compares all journal words, including unused words and completion order.
The existing owner, template, USER-table, zeroing and whole-release checks remain
in force. No table is activated or modified by the new checker.

## Validation and review

The [committed receipt](tools/user-memory/results/stage-4-inactive-boot-coverage.json)
records **all fifteen commands passing** over **989 unchanged inputs** in
**852.66 seconds**. These include **654 frontend**, **257 semantics**, five
targeted compiler, five canonical-layout and four legacy-owner checks, default
ABI/workspace/all-target checking, formatting/file ceilings, **292 user-memory**
and **83 process-memory** models. Four new interpreter tests include a missing
interior page with valid endpoints, ancestor restrictions, unsupported granules,
external pointers, USER access, bounds, rounding and both bank topologies.

The new native VM completes preparation, construction, repeated checking,
erasure and whole release, followed by initial application execution and three
exec replacements. The backing-only regression completes its corresponding
protocol. Each checks eight console-copy lines. The new kernel reconstructs
**24 objects and 919 allocated contributions**, with SHA-256
`d86386762f9842244870d593ebf3c073d2244e8110e8f93e7806085151b12a6e`.
The backing-only VM retains
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`;
the original-profile rebuild retains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.

Each coverage check walks **1,651 pages per bank**, or **3,302 page views**
across **22 role views**. The eleven page counts are
`[1, 3, 1, 1, 1, 1, 120, 1, 498, 512, 512]`.
The boot-parameter interval is **65,536–69,632**; the actual boot resource is
**668,917,760–670,954,960**. These are observed provider facts, not source
geometry or allocation authority. Both whole banks remain eight pages, with a
separately owned one-page low USER leaf. All **69,632 bytes** are erased and
released through the existing ownership protocol.

Independent replay rejects **22 new offline controls**, including missing or
changed role extents, journal completion/count/reserved-word changes and, in
each bank, absence, read-only ancestors, executable data, physical aliasing,
cache changes, external pointers and unsupported granules. Existing **23 boot**,
**24 processor**, **22 table/low-leaf** and **12 backing-only** controls also
pass. These controls alter copied records/bytes in memory; they do not mutate
guest files or execute the kernel's native refusal paths.

The conservative helper-stack sum is **2,872 bytes**, and the combined bound is
**6,968 / 8,192 bytes**. Exact imports, acyclic direct calls, frames and permitted
architectural read sites pass inspection. Peak/limit are
**1,397,522,432 / 3,095,105,536 bytes**; swap and every recorded memory-event
counter are zero. No host protection or VM boundary is changed.

The restored ordinary bootstrap root is **46,354,432**, token **[1, 5, 0]**,
whole count/alignment/pins **5/1/1**, with WP set and bank SHA-256
`b7383ffb2be791fe95bddbb16556cabea0a25599aeb8952b779e8a5ba7677405`.
Its equality to the preceding capture does not fill historical missing evidence.
Initial preparation still observes WP/NXE clear; inactive NX bits do not prove
effective hardware permissions in that context.

The receipt binds **15 tested changed inputs** and **67 native artifacts** to
hashes/lengths. Later descriptive reporting is recorded separately. The full
local run is `.cache/x86_64/inactive-boot-coverage-stage-1/run.json`; the native
build is `.cache/x86_64/process-mm-interface-regression-2685940-1794656920877257114`.
No implementation input changed after the frozen matrix. Descriptive report
updates, including the watched status file, have separately recorded hashes.
Reproduce with a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_image_backing.py linux-kernel/.cache/x86_64/NEW-OUTPUT --tables
```

Risk is high because physical table walking and permission interpretation are
security relevant. A separate single-agent review pass checks authority,
pre-read bounds, finite level/extent walks, ancestor semantics, large/small PAT
positions, exact retained facts, fatal partial failure, owner/USER exclusion,
call/stack closure and predecessor identities. The independently implemented
interpreter and interior-hole controls provide a second implementation check;
human review remains with the PR. Pinned application binaries stay unchanged.

## Remaining obligations

Successful native retained-role reachability is qualified here. Invalid-table
and journal refusals have model/offline coverage; native failure-path execution
remains an acceptance gap. The full historical compiler suite and 51-workload
third-party corpus were not rerun for this source snapshot.

These inactive checks do not qualify every future non-image role, device memory
attributes, active/restorable roots and aliases, competing writers, CPU
participation, effective WP/NXE, invalidation/publication, asynchronous/fatal
paths or continuous lifetime. Firmware inventory grants no parking or startup
authority. All protected-context gates remain required before activation.
The current serialized child/pair paths retain one process hierarchy while
swapping USER leaves and continuations; they do not clone a root bank.

Full Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance remain
unfinished. Step 4 and the original kernel-replacement goal remain in progress.

# Scoped ownership of inactive image tables

`TK-DEC-292` retains exact pinned image-table owners across ordinary boot,
userspace, exec and root restoration. The preceding immediate-discard protocol
releases its banks before that work. Source now has an explicit scoped consumer,
`memory kernel image retain tables image-tables during { ... }`, whose block
contains the required observation and independently checked boot continuation.

The provider revalidates complete storage, erases it and releases each whole
owner before the continuation's designated terminal disposition. Earlier fatal
branches retain/poison held resources. This is a systems resource protocol,
with unchanged core semantics and no general finalizer feature.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through the
[design](design/retained-image-tables.md) to
`TOPAL-SYSTEMS-KERNEL-IMAGE-RETAIN-001`, source/transition/model checking,
compiler whole-owner bindings, native retention/retirement and independent
read-only observations. Existing immediate-discard and USER-table admission
contracts remain in force. No prepared bank is activated.

## Ownership and context boundaries

The scope carries two image-derived whole banks and a separately owned low
USER leaf. Their issuer, IDs, slots, generations, counts, pins, physical extents
and stored words remain bound. Ordinary bootstrap/process roots, image, stack
and heap lifetimes remain separate. Unrelated allocator issuance may advance;
it cannot replace a retained owner or roll its generation history back.

Before entering the scope, the provider completes the original boot/context,
table-template and retained-role comparisons. Those observations then remain
historical. They do not assert that CR3, descriptors or control state stay
unchanged through userspace. Retirement revalidates the same storage owners
and complete table/zero-leaf bytes under the actual restored context, repeats
role coverage, then erases/reads back/unpins/releases every complete extent.
Future activation still needs complete context, root, mode and participant
qualification.

The source checker requires the exact context/resource and one consuming scope,
rejects ambiguous names and duplicate/interleaved consumers, and places whole
retirement after the independently checked continuation operations. No ordinary
operation follows its discard observation before terminal disposition. Existing
ordinary resources must also satisfy their completion obligations.

## Validation and review

The [committed receipt](tools/user-memory/results/stage-4-retained-image-tables.json)
records **17 leaf validation commands passing** over **998 unchanged inputs**
in **1,214.16 seconds**. Two retained execution/readback commands and the
fifteen-command predecessor matrix run through three orchestration commands.
Checks include **656 frontend**, **259 semantics**, six component, five
canonical-layout and four legacy-owner tests, default ABI/workspace/all-target
checking, formatting/file ceilings, **296 user-memory** and **83 process-memory**
models. New coverage includes two frontend tests, two semantic models, one
compiler component test and four independent owner models.

All **three native VM protocols** pass: scoped retention, immediate table discard
and backing-only preparation. Each completes initial execution plus three
successful exec replacements and eight checked console-copy lines. The retained
VM captures **33 actual syscall entries**, including **28 exec attempts** and
**five exit/exit-group entries** from the existing unchanged fixture. Its
ordinary refusal/exit cases remain part of the workload. Read-only observation
checks all retained banks and all five ordinary root/image/stack owners at each
entry, including their physical exclusion and complete pinned cells.

The syscall breakpoint derives from the actual linked provider's existing
RIP-relative LSTAR setup. ELF symbols and complete setup/entry instruction bytes
are independently checked; no debug instruction or application patch is added.
The observer stays within the existing hardware-breakpoint budget.

Seven native lifecycle phases cover seeded/zeroed/constructed/retained storage,
post-continuation validation, erasure and release. The actual allocator context
changes from **[1, 1, 5, 151992]** to **[1, 1, 204, 151992]**, while retained IDs
**2/3/4**, generations **1**, whole sizes **8/8/1** and pins **1** remain stable.
Coverage checks **3,302 page views** across eleven roles on each of three checks.
All **69,632 bytes** are erased and released with issued history preserved.

The retained kernel reconstructs **24 objects and 925 allocated contributions**,
with SHA-256
`1f44d987abb3cc2cd19e7821f900242992e10bd1be6485ff694e89db9e11f94e`.
The immediate-discard kernel remains byte-identical to the preceding stage:
`d86386762f9842244870d593ebf3c073d2244e8110e8f93e7806085151b12a6e`.
The backing-only VM retains
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`;
the original-profile rebuild retains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.

The actual restored bootstrap root is **46,428,160**, token **[1, 5, 3]**, whole
count/alignment/pins **5/1/1**, with bank SHA-256
`c46cf35dd4137c2039958ce5f9bb9ae0133b34fc7712e21ee824bb1fd0f35426`.
Its CR0 is **2,147,549,201**, with WP set, and CR4 is **32**. During static
process syscall entries, CR4 is **1,568**, matching the existing provider's
declared SIMD controls while preserving prior bits. These captures do not fill
historical observation gaps or prove protected image mappings.

The helper-stack bound is **3,224 bytes**, or **7,320 / 8,192 bytes** combined
with the existing gateway window. Exact imports, acyclic calls, frames and
permitted architectural read sites pass inspection. Peak/limit are
**2,020,220,928 / 3,276,083,200 bytes**, with zero swap use and memory events.
Host protections, VM boundaries and pinned application binaries are retained.

Independent replay rejects **21 new offline controls**, including lost phases
or events, wrong issuer/generation/pins/cells, stale context, regressed issuance,
coherent physical aliasing, changed tables and journal/order corruption. Existing
23 boot, 24 processor, 22 table, 22 coverage and 12 backing controls also pass
in the predecessor matrix. Altered saved data is not native failure execution.

The receipt binds **38 tested changed inputs** and **173 native artifacts** to
hashes/lengths. Later reporting is recorded separately. The local run is
`.cache/x86_64/retained-image-tables-stage-3/run.json`; the retained build is
`.cache/x86_64/process-mm-interface-regression-2697619-1794660878486274851`.
Reproduce using a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_retained_image_tables.py linux-kernel/.cache/x86_64/NEW-OUTPUT
```

Risk is high for affine ownership, generation and context/lifetime semantics.
A separate single-agent review checks higher-level intent, source consumption,
model ordering, exact plan/source association, actual complete owners, unchanged
USER admission, pre-read bounds, fatal ordering, call/stack closure and full
retirement. The independent interpreter and coherent-alias/generation controls
provide a second implementation check; human review remains with the PR.

Preliminary runs remain failed evidence. One observer initially selected the
later initial-user dispatcher after static-process owners were released. Later
observers incorrectly assumed four exec/exit entries and then only exec entries
before final exit. The corrected observer captures all supported entries from
the unchanged workload. Reader preflight also corrected an initial-CR4
assumption using the existing process provider's SIMD setup. A diagnostic replay
supplied a synthetic count field only in memory and was explicitly unqualified;
the final frozen VM supplies the real count. Success flags now follow all
summary checks. No kernel security guard was removed to resolve these failures.

## Remaining obligations

This stage qualifies successful scoped inactive ownership and observed complete
retirement. Internal-invalid owner/table/journal refusal paths have model/offline
coverage; native failure execution remains an acceptance gap. The historical
full compiler suite and 51-workload third-party corpus were not rerun here.

Inactive banks, event snapshots and firmware inventory grant no activation,
effective image permissions, processor startup/parking, continuous exclusion or
protected-image witness. Every active/restorable root, alias, writer, CPU,
asynchronous/fatal path, effective mode, publication/invalidation and protected
lifetime still needs complete integration. Current serialized child/pair paths
retain one process hierarchy while exchanging USER leaves and continuations;
they do not clone a root bank.

Full Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance remain
unfinished. Step 4 and the original kernel-replacement goal remain in progress.

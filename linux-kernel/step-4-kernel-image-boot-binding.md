# Actual boot and interpretation binding

`TK-DEC-289` strengthens the existing inactive-table protocol under the standing
systems authorization. Core semantics and source syntax are unchanged. The
provider now binds the actual boot adapter and retained hardware state before
preparing tables, then checks them again before discard. The prepared banks
remain inactive and are erased and released before ordinary boot continues.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through the
[design](design/kernel-image-boot-binding.md) to
`TOPAL-SYSTEMS-KERNEL-IMAGE-BOOT-BINDING-001`, fresh adapter reconstruction,
native guards, instruction/call/stack inspection and independent saved-byte
replay. The existing whole-owner and USER-table validators retain their meaning.

## Binding and native checks

The private producer freshly reconstructs the complete boot image from the
complete kernel and compares its bytes. Its record identifies boot-only jump
handoff code, retained entry translation pages, GDT, IDT and IDT pointer,
bootstrap stack, actual executable kernel/handler symbols and cache indices.
Boot-reserved storage stays retained; a handoff does not authorize reclamation.

Native guards check all **2,048 entry-table words**, **six GDT words**, **512
IDT words**, the **ten-byte IDT pointer**, and all **126 transition bytes**,
including the actual linked entry immediate. Unused table/descriptor entries
must match. Accessed/dirty changes are allowed only in applicable fields; the
null GDT descriptor stays exactly zero. Actual GDTR/IDTR, stack/code selector,
critical flags, CR0/CR3/CR4 and retained boot-resource facts must match the
supported initial profile and allocator context.

CPUID checks precede optional reads, including the general MSR-support bit and
the specific APIC/MTRR/PAT and long-mode/NX capabilities. The provider reads
EFER, APIC base, PAT, MTRR capability/default, every advertised variable MTRR
pair within its sixteen-pair bound, and all fixed MTRRs when advertised. The
same identities and interpretation state must match before discard. A private
read-only checkpoint exposes each actual MSR number/value to the observer;
the journal alone is insufficient for independent replay.

The initial direct-window cache index remains zero and the HPET/APIC views
retain index three. Independent replay checks those indices across large and
small leaves, accounting for their different PAT-bit positions. Raw attribute
observations do not themselves qualify device access, active mappings or
effective protected permissions. Source receives no register or pointer API.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-kernel-image-boot-binding.json)
records **all fifteen commands passing** over **979 unchanged inputs** in
**837.93 seconds**. This includes **654 frontend**, **257 semantics**, five
targeted compiler component tests, five canonical layout tests, four legacy
owner tests, default ABI/workspace/all-target checks, formatting/file ceiling,
**280 user-memory** and **83 process-memory** models.

The fresh native protocol completes initial application execution and three
exec replacements, with eight checked console-copy lines. Five preparation
snapshots retain all three whole owners and complete table bytes, followed by
whole erase/release. The previous backing-only protocol also completes its
ordinary application/exec workload. Its kernel identity remains
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`;
the original-profile rebuild remains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.

The new kernel reconstructs **24 objects and 899 allocated contributions**:
`3e106c3bfdeaf097655db8854223f55486d0f49cec3487ac689bec3235c49840`.
Independent replay checks **64 actual MSR reads**, eight variable MTRR pairs,
eleven fixed registers, two complete boot-data snapshots and both VM CPUs.
EFER is independently observed as **0x500**. The initial root is **0x101000**
with CR0 **0x80000011** and CR4 **0x20**. WP/NXE are clear at this early boundary;
these observations grant no protected witness. The final ordinary bootstrap
capture has WP set and actual CR3 **46,342,144**, token **[1, 5, 0]**, whole
count/alignment/pins **5/1/1**, and bank SHA-256
`76681ac35d7adf35af6480aa5e89c93e831ecc745bd56669233d8c3c2fb942f2`.
These facts concern this image and do not relabel historical missing captures.

The helper's conservative stack sum is **1,672 bytes**, including eight bytes
for balanced flags-stack use; the normal gateway window yields **5,768 / 8,192
bytes**. Exact imports, acyclic direct calls and the declared architectural
read sites pass inspection. The new component writes no control or descriptor
register. Peak/limit are **1,844,563,968 / 3,167,989,760 bytes**; swap and every
recorded memory-event counter remain zero.

The independent readers reject **23** boot-record/rehash controls, **22**
table/low-leaf/rehash controls and **12** backing-only controls. Boot controls
include unsupported mode/capability/count facts, missing/reordered/changed
MSR reads, changed retained state, resource and CPU identities, EFER monitor
disagreement, handler/adapter identity, invalid descriptor/table bits and an
altered jump target. Byte controls update their claimed digest too. They alter
saved data in memory only; the guest and retained files stay unchanged. Model
counterexamples and offline rejection do not establish native fault coverage.

The compact result binds **22 changed authority/code inputs** to their tested
hashes and **43 native artifacts** to hashes and lengths. Descriptive status
reporting after validation is identified separately. Full local evidence is
`.cache/x86_64/kernel-image-boot-binding-stage-2/run.json`; the native build is
`.cache/x86_64/process-mm-interface-regression-2666927-1794646618148345378`.
Reproduce using a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_image_backing.py linux-kernel/.cache/x86_64/NEW-OUTPUT --tables
```

Risk is high for hardware-state interpretation, memory validation and
compatibility. Single-agent implementation and a separate diff-review pass
check higher-level authority, capability ordering, complete used/unused entry
coverage, exact native identities, closed instruction/call/stack obligations,
whole owners, independent byte oracles and predecessor kernel identities.
Independent human review remains for the PR. Host protections, VM boundaries,
no-swap limits and pinned application binaries are preserved.

A preliminary candidate was refused before loading because the instruction
inspector confused an alignment NOP's CS prefix with a register read. The fix
distinguishes the actual operand and adds a regression. The first full matrix
was deliberately interrupted to add the explicit MSR-support guard; its record
remains incomplete. Neither event establishes an AI-safeguard cause.

Primary register references are [Intel SDM Volume 4](https://www.intel.com/content/dam/develop/external/us/en/documents/335592-sdm-vol-4.pdf)
and [Volume 2B, RDMSR capability requirements](https://cdrdv2-public.intel.com/782151/253667-sdm-vol-2b.pdf).

## Remaining obligations

Both-CPU snapshots establish observations, not AP parking, asynchronous-writer
exclusion or continuous lifetime. Effective WP/NXE establishment, every active
and restorable root, aliases, all mapping writers, publication/invalidation,
asynchronous entries, fatal/retirement behavior and production reconstruction
remain required before protected activation. Unsupported-state native failure
coverage remains an acceptance gap. The historical full compiler suite and
51-workload third-party corpus were not rerun for this source snapshot.

Full Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance remain
unfinished. Step 4 and the original kernel-replacement goal remain in progress.

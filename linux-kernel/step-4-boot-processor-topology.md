# Initial firmware processor inventory

`TK-DEC-290` adds bounded native ACPI processor discovery to the private
inactive-table boot binding. The kernel now identifies the initial firmware
processor inventory itself. Source syntax and core semantics are unchanged;
the provider exposes no firmware pointer, table format or controller identifier
to source. This stage does not start, reset or park another processor.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through the
[design](design/boot-processor-topology.md) to
`TOPAL-SYSTEMS-BOOT-PROCESSOR-TOPOLOGY-001`, native discovery and retained-copy
decoding, closed component inspection, read-only VM captures and independent
firmware replay. Existing allocator/USER validators and predecessor profiles
retain their meaning.

## Discovery and retained decoding

The supported legacy boot path searches EBDA before the BIOS window on the
required boundaries, validates RSDP signature/revision/checksums, and follows
RSDT or XSDT to one unambiguous MADT. The search uses its declared EBDA/ROM
windows. SDT reads require retained firmware ranges in the initial direct
window, excluding allocator-eligible RAM and conflicting claims. Root entries,
table lengths, record walks and retained
storage have explicit bounds; unknown interrupt/processor record forms fail
closed. Unaligned firmware table addresses are supported.

The provider retains the discovery bytes in its own bounded copy, validates
the copied lengths/checksums and decodes those same bytes. Separately captured
headers must agree with their complete table. It compares every relevant byte
again before discard, so a preserved checksum cannot substitute for equality.
This establishes internal snapshot consistency, not atomic firmware capture or
continuous firmware ownership.

Local APIC/x2APIC records preserve their kind, firmware UID, native identity and
enabled/online-capable status. Unusable records are ignored. Relevant UIDs and
controller identities must be unique, and exactly one enabled record must
identify the actual BSP. The provider checks the existing UC controller view
before its MMIO read and compares the result with CPUID's initial identity.
Discovery yields inventory facts; processor participation and lifetime remain
separate obligations.

A general private read checkpoint replaces the MSR-only checkpoint. Its three
arguments distinguish actual MSR, controller-ID and CPUID results. One read-only
hardware breakpoint captures these values before journal storage, retaining the
existing debugger breakpoint budget. CPU controls, firmware bytes and pinned
application instructions are not modified by observation.

## Validation and review

The [committed result](tools/user-memory/results/stage-4-boot-processor-topology.json)
records **all fifteen commands passing** over **985 unchanged inputs** in
**836.65 seconds**. This includes **654 frontend**, **257 semantics**, five
targeted compiler component checks, five canonical layout checks, four legacy
owner checks, default ABI/workspace/all-target checks, formatting/file ceiling,
**288 user-memory** and **83 process-memory** models. Eight new models cover
legacy and extended discovery, flags/identities, provenance/conflicts, bounded
walks, corrected checksums and inconsistent retained views.

Both fresh native protocols complete ordinary initial execution and three exec
replacements, each with eight checked console-copy lines. The new image
reconstructs **24 objects and 915 allocated contributions**; its kernel SHA-256 is
`05dcaa5f8fdf2523ef9dd7cc67adc35af4d7bd5adde00b6363cac55c9be07f8c`.
The previous backing-only VM retains
`064058b5c8fb800137cf055b971dbc1cf258ab3729ba93cfda005bc4ea805b19`;
the original-profile rebuild retains
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.

Actual firmware uses RSDP revision **0**, a **56-byte RSDT** with **five root
entries**, and a **128-byte MADT**. The inventory contains **two enabled Local
APIC records**, with UID/controller pairs **0/0** and **1/1**, and no online-only
record. The BSP identity is **0**. Both native MMIO reads return **0**; both
CPUID EBX observations are **133,120**, with initial identity zero. Independent
replay checks all four identity reads, all **386 retained firmware bytes** and
their unused storage, two full capture sets and the VM CPU count. The preceding
boot checks still verify 64 MSR reads, actual boot bytes and cache indices.

Independent readers reject **24** processor discovery/read/retained-copy
controls, alongside **23** preceding boot controls, **22** table/low-leaf controls
and **12** backing-only controls. New cases include missing/reordered/changed
identity reads, wrong counts/identities, lost firmware provenance, duplicates,
invalid processor flags/record lengths, unsupported formats and corrected-
checksum changes. A semantically valid, checksum-correct firmware change at
discard still fails retained equality. These are offline controls over copied
records/bytes; they do not inject guest state or establish native failure-path
execution.

The conservative helper stack sum is **2,552 bytes**; with the normal gateway
window, the bound is **6,648 / 8,192 bytes**. Exact imports, acyclic direct calls,
frames and architectural read sites pass inspection. The new component writes
no control or descriptor register. Peak/limit are
**1,305,051,136 / 3,142,836,224 bytes**; swap and all recorded memory-event
counters remain zero.

After ordinary application completion, the actual restored bootstrap root is
**46,354,432**, with token **[1, 5, 0]**, whole count/alignment/pins **5/1/1**,
WP set, and bank SHA-256
`b7383ffb2be791fe95bddbb16556cabea0a25599aeb8952b779e8a5ba7677405`.
The prepared eight-page banks were erased/released without activation. These
captures concern this new image and do not fill historical observation gaps.

The compact result binds **16 changed authority/code inputs** to their tested
hashes and **67 native artifacts** to hashes/lengths. Later descriptive
reporting is recorded separately. The full local run is
`.cache/x86_64/boot-processor-topology-stage-2/run.json`; the native build is
`.cache/x86_64/process-mm-interface-regression-2679862-1794654712029229024`.
Final staged review removed only surplus EOF blank lines in five new files.
The receipt records their tested/formatted hashes separately. Three Python ASTs
remain identical, and fresh LLVM verification/compilation reproduces the exact
native component object from the frozen VM build. The VMs were not rerun for
this formatting-only cleanup.
Reproduce with a fresh output directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_image_backing.py linux-kernel/.cache/x86_64/NEW-OUTPUT --tables
```

Risk is high for firmware parsing, physical access and hardware interpretation.
A separate single-agent review pass checks higher-level authority, native read
bounds, exact retained-copy decoding, checksum/header consistency, complete
record walks, identity/flag handling, closed instruction/call/stack obligations,
whole-owner invariants and predecessor identities. Independent human review
remains for the PR. Host protections, resource limits, VM boundaries and pinned
application binaries are retained.

The preliminary VM succeeded, but its reader correctly failed when a GDB memory
read returned `ffffffff` for the APIC ID register. QEMU's local-APIC display
labels CPU index, so it was not substituted for the register value. The native
read checkpoint now supplies the actual MMIO/CPUID results. The first full
matrix was deliberately interrupted during its last regression to change native
decoding from repeated firmware reads to the validated retained copy. The final
matrix reruns all commands after that change. These diagnostics do not explain
any earlier AI safeguard interruption.

Format source: [ACPI 6.6, discovery and MADT](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html).
Observer limitation: [QEMU 10.2.2 local-APIC display](https://github.com/qemu/qemu/blob/v10.2.2/target/i386/cpu-dump.c).

## Remaining obligations

The native VM covers the legacy RSDP/RSDT path. Extended XSDT, mixed x2APIC,
online-capable/unusable records, malformed/unsupported input and changing-view
refusals have model/offline coverage in this matrix; native failure coverage
remains an acceptance gap. The historical full compiler suite and 51-workload
third-party corpus were not rerun for this source snapshot.

Inventory and CPU snapshots grant no startup, parking, hotplug, scheduling or
all-CPU lifetime authority. Every active/restorable root, alias, writer,
effective permission mode, publication/invalidation, asynchronous/fatal path
and retirement still needs complete integration before protected activation.
The current serialized child/pair implementation retains one process hierarchy
while swapping USER leaves and continuations; it does not clone a root bank.

Full Linux UAPI, DTS, distribution, Docker/Podman and VM-host acceptance remain
unfinished. Step 4 and the original kernel-replacement goal remain in progress.

# Boot-owned storage for processor startup

`TK-DEC-295` implements a native prerequisite of the explicit participant entry
contract: a whole boot-owned low page, qualified before writing and retained
through ordinary execution. The [systems design](design/boot-startup-storage.md)
derives the owner from the complete selected adapter handoff rather than an
apparently unused physical address or an allocator reservation alone.

`TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-COMP-SYSTEMS-KERNEL-IMAGE-001` trace through
`TOPAL-SYSTEMS-BOOT-STARTUP-STORAGE-001` to the
[adapter construction](../src/topal-compiler/src/systems_boot_startup.rs),
[native checks](../src/topal-compiler/src/runtime/kernel_boot_startup.ll),
[independent byte oracle](tools/user-memory/boot_startup_storage.py) and
[validation](tools/user-memory/run_boot_startup_storage.py). The change is
systems-only under the standing chat authorization. Core language meaning and
the public participant-initialization refusal remain unchanged.

## Handoff and ownership

The opt-in private adapter declares 8,192 loaded setup bytes; its second whole
page carries a deterministic loaded pattern. The pinned QEMU loader/option-ROM
route copies the declared setup extent. Default adapters keep their original
2,560-byte setup layout and payload offset.

Native checks bind the actual aligned low setup base, exact supported header,
whole loaded pattern, bounded raw E820 claims, complete RAM coverage and absence
of conflicting claims. They exclude firmware/BDA/EBDA, command-line/extended
inputs, continuation/stack, high image and boot-resource storage. Existing
allocator-prefix exclusion preserves the boot owner's lifetime; it does not
manufacture a frame token or permit release through another issuer.

After qualifying both inactive banks' actual supervisor RW/NX identity views,
the provider writes and reads back one complete XOR probe, restores and checks
the original page, then retains it. It revalidates the full boot-parameter page,
owner identity, firmware table, startup bytes and both inactive views before
table retirement. Historical entry-root fields are not assertions that the
ordinary execution root remained unchanged. Failures retain storage and enter
fatal; this protocol has no reclamation operation.

## Native evidence and review

The passing VM run observes the page at **0x11000** from an actual **0x10000**
setup base. Independent replay checks **44 complete snapshots**, including the
three loaded/probe/restored checkpoints, seven table-lifetime phases, **33
ordinary syscall entries** and final root restoration. The workload completes
three successful exec replacements and eight checked console transfers.

The startup reader also rejects **13 altered saved-record controls**, including
rehashed page/parameter changes, stale generation, coherent aliasing, missing
observations, changed EBDA, incompatible mapping permissions and reclamation.
Those controls are offline evidence; they do not claim native fault injection.
The existing retained-table reader independently passes its 21 controls and
checks full 69,632-byte table/leaf erasure and release. The startup page remains
retained after those separate allocations retire.

The native kernel is 1,849,712 bytes, SHA-256
`0678e5f55ee880714135a7efc51cd67aab05a26e1d647cd09c5d11424c71eadd`.
The closed component bounds helper stack at **3,672 bytes**, plus the unchanged
4,096-byte gateway window, for **7,768 of 8,192 bytes**.

The [receipt](tools/user-memory/results/stage-4-boot-startup-storage.json)
records **19 leaf commands passing over 1,009 unchanged inputs** in
**1,430.21 seconds**. It includes two new component tests, sixteen boot-adapter
tests, the separately executed native VM test and both independent readers.
The fourteen-command predecessor matrix passes 661 frontend and 263 semantics
tests, participant/compiler/provider/owner/layout regressions, default ABI,
workspace/all-target checking, Cargo/included-file formatting, Rust file ceilings
and **323 user-memory plus 83 process-memory models**. Three old backing VM
tests remain ignored; the new VM test executes separately from its ignored
appearance in ordinary unit groups.

The legacy build-only kernel remains 1,767,200 bytes, SHA-256
`28057d5606b6c59f704e861b2e22d0b6dd04e79f9d07ff4933546695214204c1`.
The complete cohort peaks at **2,088,579,072 / 3,168,088,064 bytes**, with zero
memory events or validation swap. The unchanged two-CPU VM, host protections
and resource wrapper remain in force.

Risk is high because the change writes low boot memory and preserves ownership
across root/lifetime transitions. Separate single-agent self-review checked
the higher-level ownership/publication/fatal rules, pinned loader copy route,
native bounded reads and exclusion checks, exact initial/later binding, both
inactive table views, full page restoration, unchanged default selection and
independent actual-byte readback. Model and observation tools remain distinct
from native enforcement. A preliminary full run failed before VM execution
because new helper functions lacked freestanding attributes and produced an
unqualified unwind section. The helper now uses the existing attributes, and
its component test invokes the actual publication section validator. The
validator and host protections were retained.

Reproduce with a fresh lab directory:

```sh
rtk scripts/run_bounded.py -- python3 -B linux-kernel/tools/user-memory/run_boot_startup_storage.py linux-kernel/.cache/x86_64/NEW-OUTPUT
```

## Remaining work

The next participant increment needs actual compiler-generated entry with owned
root/stack/mailbox/exception support, target-local capability/mode/identity,
ordered acknowledgment, bounded dispatch/arrival and retained failure/late-entry
behavior. This increment sends no startup request and supplies no arrival,
continuous exclusion or protected-context witness. Existing inactive banks must
remain independent of any future participant root because they retire normally.

Unsupported handoffs, conflicting firmware claims, failed native probes and
changed live-page fatal paths have not been exercised as negative VM cohorts
in this stage. Offline controls do not close those native coverage gaps; they
remain acceptance blockers alongside the participant and protection work.

Complete all-root/alias/writer/CPU protection, native asynchronous/fatal coverage
and production qualification remain required. Step 4 and the original full
Linux interfaces, Device Tree, distribution, Docker/Podman and VM-host acceptance
are unfinished.

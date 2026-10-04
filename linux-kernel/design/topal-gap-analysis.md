# Topal kernel-readiness gap analysis

## Conclusion

Linux-compatible kernel implementation in Topal is feasible as a language and
toolchain project, but not with the currently qualified compiler/profile.
Topal's existing design supplies useful foundations; the missing pieces are
concentrated at privileged entry, shared kernel synchronization, address-space
control, fault recovery, context transfer, and kernel artifact publication.

The gaps are not solved by writing ordinary library code because current
source cannot create the required authority or express the required machine
transition. They also do not justify a general assembly or unsafe-pointer
escape hatch. The proposed systems profile supplies narrow elements, after
which most of the kernel remains ordinary typed Topal modules.

## Reusable without changing meaning

| Existing Topal design | Kernel use | Constraint retained |
| --- | --- | --- |
| semantic values separated from layouts | UAPI, ELF, firmware, packet, descriptor, and register encoding | external bytes never become values without validation |
| addressed storage and resource identity | kernel memory and MMIO locations | numeric equality does not imply identity or authority |
| effects parameterized by resources | VFS, device, clock, security, and observation ordering | effects expose dependency, not ambient permission |
| capabilities and constructed contexts | subsystem authority and boot composition | architecture facts cannot mint runtime authority |
| resource ownership and regions | allocations, device sessions, mappings, queues | cleanup and non-escape remain explicit |
| task protocols and typed messages | selected services, work queues, control planes | ordinary task semantics are not stretched into hardware entry |
| architecture models and evidence seam | legality of target lowering and board facts | models stay declarative and authority-free |
| contracts, information flow, and trace identity | invariants, credential/policy flow, differential evidence | unverified claims cannot discharge protected obligations |

These foundations are design-level. Some remain planned in the compiler
coverage ledger and therefore also require implementation before kernel use.

## Mandatory semantic additions

| Gap | Why a library alone is insufficient | Proposed element |
| --- | --- | --- |
| boot/trap/interrupt/syscall entry | physical entry violates ordinary call ABI and creates privileged authority | `TK-ELEMENT-ENTRY-001` |
| external event and concurrent choice | current schedule equivalence cannot expose interrupt/coherence/scheduler winner selection | `TK-ELEMENT-OBSERVATION-001` |
| privileged state transitions | ordinary source cannot authorize or lower control/MMU/interrupt operations | `TK-ELEMENT-MACHINE-001` |
| distinct address spaces and page mappings | current provisional locations do not construct/activate translation state | `TK-ELEMENT-ADDRESS-001` |
| recoverable user-memory faults | failure arises at a generated machine access and recovery target | `TK-ELEMENT-USER-ACCESS-001`, `TK-ELEMENT-FAULT-001` |
| shared atomic kernel state | current design intentionally exposes no source atomics/locks | `TK-ELEMENT-ATOMIC-001` |
| interrupt/preemption scopes | state is CPU-local, nested, affine, and not an ordinary mutable Boolean | `TK-ELEMENT-CRITICAL-001` |
| CPU/device/DMA ordering | portable task order alone cannot establish hardware visibility | `TK-ELEMENT-FENCE-001` |
| scheduler context switching | control resumes a suspended stack rather than returning from a normal call | `TK-ELEMENT-CONTEXT-001` |
| device/DMA ownership transitions | asynchronous agents and IOMMU/cache state outlive ordinary evaluation | `TK-ELEMENT-DEVICE-001` |
| kernel allocation/static placement | current runtime/artifact assumes a Linux process and publisher | `TK-ELEMENT-STORAGE-001` |
| bootable kernel output and fatal path | current compiler emits a Linux static PIE using syscalls | `TK-ELEMENT-ARTIFACT-001`, fatal disposition |

## Required compiler and toolchain work

Even after semantics are approved, the current compiler must gain:

1. a distinct x86-64 kernel target/profile and architecture-provider
   qualification;
2. systems-feature parsing and semantic checks, including context and affine
   capability restrictions;
3. checked lowering for entry, privileged, atomic, address, fault, transfer,
   MMIO/DMA, and context operations;
4. a freestanding runtime with no Linux syscall or host allocation dependency;
5. kernel code/data/per-CPU/static placement and qualified relocation model;
6. backend-generated x86-64 entry/return/context stubs with unwind/debug data;
7. atomic link plus image packaging for the Step 3 boot path;
8. artifact inspection for sections, relocations, symbols, instructions,
   undefined dependencies, stack/entry rules, and provenance; and
9. interpreter/model transitions or explicit model-only diagnostics for every
   new element.

The existing Linux application target remains separate. Kernel work must not
quietly modify `x86_64-unknown-linux-gnu` into a target with two incompatible
entry/runtime meanings.

## Work that remains libraries and kernel modules

Once the mandatory elements exist, allocators, schedulers, synchronization
algorithms, VFS, filesystems, network protocols, driver frameworks, firmware
parsers, PCI/virtio drivers, namespaces, containers, syscall/UAPI adapters, and
KVM are implementable as Topal libraries/modules. Their size and difficulty
are substantial, but they do not require a new language construct merely
because Linux implements them with C-specific techniques.

Some algorithms may reveal a missing proof or progress facility during
formalization. Such a discovery returns to the protected design process; it is
not permission to introduce an unchecked intrinsic locally.

## Feasibility gates

| Gate | Evidence needed before claiming success |
| --- | --- |
| semantic | approved systems profile with formal rules for every mandatory element |
| compiler | negative and model tests plus inspected freestanding x86-64 objects |
| machine | boot/entry/fault/interrupt/context/memory tests under pinned QEMU |
| kernel core | SMP, allocator, scheduler, user transfer, VFS, time, and teardown stress evidence |
| ABI | every source/runtime inventory item has a disposition and differential evidence |
| application | pinned static/dynamic userspace corpus and distribution prompt |
| container | separate rootful/rootless Docker and Podman profiles |
| virtualization | QEMU software host, then separately qualified KVM API and optional nested profile |

Therefore “can Topal implement the kernel?” currently has a qualified answer:
the design has a credible route, but Topal is missing mandatory approved
semantics and qualified backend support before kernel code can begin safely.

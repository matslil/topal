# Boot participant arrival

`TK-DEC-293` adopts a systems-only arrival protocol under the standing chat
authorization. It refines `TOPAL-REQ-SYSTEMS-KERNEL-IMAGE-001` and
`TOPAL-SYSTEMS-BOOT-PROCESSOR-TOPOLOGY-001`. Core language semantics and current
executable profiles keep their meaning. This stage specifies and models the
protocol; source spelling, compiler lowering and native startup are not yet
implemented or qualified.

## Why inventory is insufficient

Protected image publication must account for every relevant processor and
later writer. A processor count, firmware record, successful startup request,
controller delivery completion or debugger snapshot cannot establish that a
particular processor reached the required kernel entry. An arrival also cannot
establish that the processor will remain excluded from later incompatible work.
These are separate facts with separate lifetime obligations.

Use an explicit boot-participant initialization resource and a provider-selected
typed participant entry. Source will name the operation's purpose and lifetime,
without exposing controller IDs, startup vectors, physical addresses, native
instructions or architecture register state. Initialization is opt-in; preparing
or retaining inactive image tables does not implicitly start processors.

## State and ownership

Bind one initialization to the exact provider, boot domain, qualified image,
fresh non-reused epoch and complete retained participant inventory. Include
the bootstrap processor. An enabled participant cannot disappear because it
has not responded. Runtime-enableable records require an explicit provider
qualification; the first native profile may refuse them before any effect.
The model refuses them rather than inventing their availability.

Before arming an entry, retain complete live pinned owners for its entry code,
translation root, stack, mailbox and exception/descriptor storage. Owner issuer,
identity, generation, extent and role stay bound. Shared code/root owners are
permitted only where the entry contract explicitly permits sharing. Stacks,
mailboxes and exception storage are distinct for each participant. Root sharing
does not authorize concurrent table mutation. This arrival model relies on the
separate complete-table/image validation and does not reimplement it.

All participants are pending before the first possible hardware-visible entry
effect. A secondary processor must have a matching startup request before its
entry is accepted. The bootstrap processor enters through its separately
qualified current-context route. A request records dispatch only. Each target's
actual entry must bind the same session, expected participant identity, entry,
root/stack/mailbox/descriptor owners and supported local mode. Another CPU's
capabilities or the bootstrap state cannot supply those facts.

Entry and acknowledgment are distinct transitions. The target publishes its
acknowledgment after entry validation and required initialization/visibility;
the coordinator observes it with the matching acquire/completion protocol.
Only that target's matching acknowledgment clears its pending obligation.
Duplicate, unsolicited, foreign or stale-epoch entry/acknowledgment is a
protocol failure, never evidence for another pending participant. The portable
model treats entry and acknowledgment events as provider facts; supplying
Python records cannot establish that a native CPU executed them.

The target may enter and acknowledge before the startup call returns. Record
dispatch completion separately; a failed or missing result cannot become
success because the target already acknowledged. Completion requires every
expected participant, including the bootstrap processor, to have acknowledged,
all startup requests to have completed successfully, and the retained owners/
inventory to remain exact. It yields arrival facts only. All entry and supporting storage remains
retained for the boot-domain lifetime. Arrival permits no scheduler execution,
protected-image activation, hotplug or storage reclamation by itself.

## Failure and later execution

Preparation may cancel and return its owners only before arming makes entry
possible. After arming, even a failed startup call or timeout may race a delayed
target. Do not reclaim its entry code, root, stack, mailbox or exception storage.
Retain/poison the whole affected domain and enter its qualified fatal path.
Late entry cannot revive a failed session or consume a newly issued epoch.
There is no retry/reuse or post-arm reclamation operation in this initial
protocol. A future stop/reclaim protocol needs independent acknowledgments and
architecture maintenance obligations for every possible reference.

The native coordinator must use a qualified bounded deadline, including startup
dispatch and target arrival. Timeout is failure, not evidence that a CPU is
absent. Membership changes invalidate the binding. Faults during entry or after
arrival retain/poison the domain; an earlier arrival receipt cannot survive as
current exclusion or protection authority.

Before protected execution, qualify every participant's current and restorable
roots, effective interpretation controls, asynchronous/exception/fatal paths,
firmware and device/DMA assumptions, all future startup/IPI/root writers, and
publication/maintenance completion. Waiting with interrupts masked or executing
a halt instruction cannot discharge NMI, firmware or device obligations. The
existing image lifetime and USER-admission requirements remain unchanged.

## Architecture mapping and rejected alternatives

| Provider | Native work needed behind the same entry/arrival contract |
| --- | --- |
| x86-64 | Qualified firmware inventory; owned low startup storage; supported INIT/startup route and dispatch deadlines; target-local capability/mode setup; owned entry/stack/exception path; actual identity/epoch acknowledgment |
| AArch64 | Supported firmware CPU activation and entry context; target-local exception level, translation/cache and stack setup; actual entry acknowledgment |
| RISC-V | Supported hart activation and opaque entry context; target-local privilege/translation/interrupt setup; actual entry acknowledgment |

Intel's ICR completion describes sending. Its startup algorithm separately
requires synchronization with executing application processors. Arm PSCI
CPU_ON is asynchronous and supplies an entry/context pair. RISC-V SBI HSM also
permits hart_start to return before target execution, with a specified initial
register/translation state. These differences motivate a typed entry and fresh
session, rather than a portable raw startup instruction.

Research sources are [Intel SDM 253668-085US, sections 12.6.1 and 10.4](https://cdrdv2-public.intel.com/835754/253668-sdm-vol-3a.pdf),
[Arm DEN0022D, PSCI 1.1, sections 5.6.2 and 6.6](https://documentation-service.arm.com/static/5f905c71f86e16515cdc1fd0),
and [RISC-V SBI HSM](https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/src/ext-hsm.adoc).
These are architecture research inputs, not a qualification of the latest
native provider, a selected instruction sequence or timing constants.

The pinned test VM's two CPUs stay in scope. Reducing its topology would leave
the original obligation unanswered. Relying on firmware parking could be valid
only with a qualified complete handoff and later-entry contract. SeaBIOS source
has a post-initialization halt loop, but that fact alone does not bind actual
firmware bytes, all entry paths or later writers. This stage does not adopt a
firmware parking certificate. Cooperative entry supplies the explicit target
arrival needed by the selected protocol; continuous exclusion remains separate.

## Evidence and implementation sequence

`tools/user-memory/boot_participants.py` models the protocol and exact retained
owner checks. Its tests exercise interleavings, stale/duplicate acknowledgments,
late arrival, ownership changes, cancellation and partial failure. Model inputs
describe provider observations; the model neither discovers native participants
nor constructs language, allocation or hardware authority.

Next connect an explicit systems operation and typed entry through checked IR,
semantics and the compiler; implement owned x86 startup storage and target-local
entry; observe actual per-CPU arrival and failure behavior in the unchanged
two-CPU VM. Keep native asynchronous/fatal and continuous-lifetime gaps as
acceptance blockers. Then integrate protected roots across every existing
producer and restoration path. Full Linux interfaces, distribution userspace,
Docker/Podman and VM hosting remain the original acceptance goal.

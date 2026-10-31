# Resumable static-process pair

Status: systems-only refinement adopted under standing chat authorization,
`TK-DEC-255`; implementation and qualification pending.

## Purpose and alternatives

The [first child implementation](serialized-static-process-children.md) cannot
return the parent before child exit. A child that needs its parent to make
progress therefore cannot complete. This refinement lets both independently
owned static processes become runnable and hands execution between them at
native `sched_yield` and blocking child-wait boundaries. It preserves the
single additional child limit and existing static mapping/executable profile.
It establishes cooperative progress, without claiming timeslice fairness,
preemption of arbitrary user instructions, general blocking I/O or threads.

Eager separate address spaces remain the general implementation direction.
For the current owned resources, exchanging the active private state with one
inactive snapshot avoids another complete image reserve and exercises the same
lifetime boundaries in both directions. Repeatedly returning a zero yield
without making the runnable counterpart eligible cannot qualify this refinement.
Keeping the parent parked until child exit remains the preceding profile.

## Ownership and scheduling

The retained pair owner records the actual active process, its one inactive
counterpart, the inactive native return value, and whether the parent is waiting
for this exact child. Integer PIDs do not construct a runnable context. Creation
initially selects the child; its inactive parent owns the pending positive fork
return. A yield selects the counterpart if it is runnable. The yielding process
owns a zero return when it is next selected. A yield with no runnable counterpart
returns zero locally and ignores unused native arguments.

An exchange preserves and reads back every private mutable image, reserved heap,
stack, leaf permission, native return register, TLS/extended-state value,
descriptor slot/flag, cwd, signal and thread registration. It invalidates stale
translations before user return. Both contexts retain their own image generation
across unrelated exec. Shared descriptions, clock observations and entropy history
remain with the pair's shared owners. No inactive key/counter copy may revive old
entropy state. Scratch copies are erased/read back before the other user runs.
No live kernel continuation or partially committed user transfer is suspended.

The description store belongs to the pair, independently of snapshot bytes.
Either process may own any of its thirty-two cells. Releasing an inactive snapshot
must not erase descriptions still referenced by the active parent. This replaces
the preceding implementation's child-only interpretation of its second bank.
One parent table of sixteen slots can inherit/retain second-bank descriptions
through child exit, wait and the next creation; capacity is determined by both
tables' live slots, rather than by a bank or a process name.

## Wait, exit and teardown

The existing native selector/options/rusage admission remains. A child cannot
wait for its parent. For a live matching child, WNOHANG returns zero unchanged.
A blocking parent wait atomically records its exact relationship, parks its
native retry continuation and selects the runnable child. A waiting parent is
not eligible at a child yield. Child exit retains status, releases its own
references and resumes the parent's exact pending continuation. Status output
is checked in the restored parent mapping before once-only reaping; an invalid
output returns EFAULT with the exited relationship retained for retry. No stale
child mapping or previously checked pointer grants parent publication authority.

If the parent was runnable, child exit resumes its pending fork or yield return
instead. Normal child exit never publishes the outer owner's terminal marker.
Parent termination tears down the entire prepared pair, releasing inactive child
references and snapshot storage before active parent descriptors and the outer
owner are returned. This is outer-owner teardown, not a claim of general orphan
reparenting or global Linux init-exit behavior. All retained candidates, native
returns, waiting identities and private bytes are erased/read back at lifetime end.

## Qualification, portability and remaining architecture work

Independent native witnesses must execute repeated child-to-parent and
parent-to-child yields while both are live; WNOHANG before child exit; blocking
wait, failed output/retry, child exec after a handoff, private data/stack/TLS/SIMD
restoration, shared offsets and entropy history, references in both banks, and
parent termination with a still-live inactive child. A controlled corrupted
inactive state must reach an exact independent failing comparison. Existing
unmodified application and library evidence must remain valid on the new kernel.

Risk is high. Review must audit exchange intervals and readbacks, private/shared
separation, wait eligibility, native return values, cross-generation publication,
both descriptor banks, exec and terminal ownership. The state model and the
actual independent user/GDB witnesses cover different failure classes.

The semantic suspended-context/relationship contract applies to AArch64 and
RISC-V. Their providers need native register and extended-state evidence,
translation invalidation and ABI-specific retry adaptation. The x86 mechanism
uses qualified SYSCALL continuations; its instruction width and return registers
are private provider details. No core-language or source-assembly change is made.

The existing bounded provider still carries substantial Linux policy. This
increment does not qualify the Step 2 requirement for ordinary Topal scheduler
modules or resolve general/persistent kernel library allocation. Those remain
explicit in [Step 4 status](../step-4-status.md), together with preemptive user
scheduling, IPC, writable storage, distribution, containers and VM hosting.

# Step 4: resumable parent and child

`TK-DEC-255` lets both independently owned static processes run before the
child exits. Native yield selects an eligible counterpart; blocking child wait
parks the parent until matching exit. This advances the
[Step 4 implementation](step-4-status.md). It does not establish general user
preemption, scheduling fairness, blocking I/O or complete Linux compatibility.

The [design](design/resumable-static-process-pair.md),
`TOPAL-SYSTEMS-STATIC-PROCESS-PAIR-001` and the preceding child/registration/wait
rules define this systems-only refinement. Core-language meaning is unchanged.
The existing native provider still carries Linux policy; ordinary Topal
scheduler modules remain an explicit Step 2/Step 4 obligation.

## Owned exchanges and wait

Creation initially selects the child and retains the parent's pending positive
fork return. A yield exchanges the active private process with its exact owned
inactive counterpart. The yielding continuation owns a zero result when resumed.
A yield with no runnable counterpart returns zero locally. Shared file
descriptions, clock history and entropy history never exchange with private state.

The provider exchanges complete mutable image, all reserved heap pages, both
stack pages, both leaf tables, private descriptor table/signal/cwd state and the
private native context. Each exchanged qword is read back at both destinations.
It separately retains/restores hardware FS and the complete legacy extended-state
record, erases/readbacks its aligned temporary record and invalidates translations
before user return. Inactive entropy history is represented by zeros; the active
generator and delivery counter remain with their shared owner. Owner/hierarchy
aliases, process identity and native continuation extents are checked before
activation. No live kernel stack continuation or partial transfer is suspended.

The existing **253952-byte** supervisor backing suffices. Metadata records the
parent's wait eligibility and the inactive native result. The second sixteen-cell
description bank now belongs to the shared store rather than to a particular
child. Either process may retain any of the **32** descriptions. Snapshot erasure
excludes this bank; child exit releases only child references, and parent exit
checks that both banks have no remaining references after releasing all owners.

For a live matching child, WNOHANG returns zero. Blocking wait validates the
native request, records the exact wait identity, parks the parent and selects the
child. A child yield cannot select a parent waiting for that child. Exit restores
the parent's pending fork/yield return or native wait retry. The x86 provider
adapts retry at the admitted SYSCALL site; it retains no checked pointer authority
from the child mapping. Status publication is checked in the restored parent
generation, and EFAULT retains the exited child for retry. Reaping remains once-only.

Parent termination releases the inactive child's references and private state
before active parent resources and the outer owner return. This qualifies whole
prepared-pair teardown, without claiming general orphan reparenting or Linux's
global-init behavior.

## Executed evidence

The kernel SHA-256 is:

```text
6109342b52ee279f66980d97801635037faa2079b8d7325fb54e81b954a05b17
```

Artifact/root/provider/object/boot/gate revisions are
`231 / 220 / 221 / 213 / 131 / 231`.

The independent [user witness](labs/qemu/x86_64/static_pair_checks.py) executes
actual native calls through the same shared services as applications. It runs
three additional children after the preceding four child cases. The first pair
alternates repeatedly, changes private data/stack, FS, XMM and a live x87 value,
and observes shared file offsets as evidence that the counterpart progressed.
It checks WNOHANG while the child lives, capacity, a blocked wait with inaccessible
status, EFAULT after exit, valid retry and repeated reap. A waiting parent remains
ineligible while its child yields.

The next pair keeps the original file descriptions in its inactive child while
the parent closes its own file slots and independently opens descriptions from
the second bank. Those parent descriptions survive child exit and reap, then
are inherited by another child and retain shared offsets through another blocking
wait. The original caller state and descriptor aliases are restored afterward.

The [GDB harness](labs/qemu/x86_64/pair_gate.py) and
[record](labs/qemu/x86_64/results/topal-static-process-pair-gate.json) observe
**13 resumptions**, two blocking waits, two local child yields while the parent
waits, second-bank retention, and complete retired snapshot/terminal erasure.
Every observed resumption checks all captured private backing bytes, page
permissions, general registers, hardware FS, all sixteen XMM registers, MXCSR,
x87 values/control/status/tags, thread registrations and private descriptor state.
Shared history and reference counts remain with their owners.

A separate positive run executes the digest-pinned, unmodified BusyBox shell
and external child. Explicit debugger input adds one native yield at the child's
clone-return SYSCALL site without changing binary bytes. Its eventual zero result
is the original clone child result. The parent continues normally, blocks in wait,
and the child resumes, execs and exits. Three observed resumptions preserve full
private state and the evolved entropy history. The ordinary external corpus runs
separately without this injected yield.

Another controlled positive run terminates the parent while its last child is
inactive and both share second-bank descriptions. The normal terminal path
releases inactive and active references, all private storage, thread registrations
and library bindings. The record labels the injected native exit explicitly and
does not claim the later application-output corpus ran in this case.

A separate negative run changes one inactive parent data byte before its first
resumption. The independent user comparison observes `0x12345679` instead of
`0x12345678`, unequal flags and the exact intentional failure loop. The observer
rejects this input before completion. Timeout alone is not accepted, and the
counterexample claims neither normal completion nor reclamation.

The retained regression matrix contains **50 passing physical VM runs on this
one kernel**: three clean environment modes, 31 unmodified external workloads,
five RTC cases, three controlled timing cases, deadline injection, finite/numeric
library debugger runs, two preceding child debugger runs and the three pair runs.
A reordered-Enum kernel is an additional positive run. Five separately retained
expected rejections cover wrong finite/numeric policy, the historical deadline
predecessor, the preceding corrupted child snapshot and the corrupted pair snapshot.

Validation includes the bounded workspace suite, 207 compiler library tests,
113 semantic-model tests, twenty toolchain lab contracts and four reference-VM
contracts, formatting, whitespace, relative links and 248 unique normative
systems rules. The model distinguishes runnable and waiting ownership, shared
history, live WNOHANG, suspension, failed publication/retry, once-only reaping
and active/inactive termination.

## Review and remaining scope

Risk is high across private/shared state, native return adaptation, TLS/extended
state, wait eligibility, exec, both descriptor banks and lifetime termination.
Self-review traced the authority chain, every exchange interval/readback, exact
resource aliases, wait/return ownership and all reclamation paths. Model tests,
independent actual user instructions, full-state debugger observations, controlled
native inputs, unchanged applications and a precise corruption counterexample
cover distinct risks. No third-party source is imported and pinned binary inputs
remain outside version control.

The implementation still qualifies two processes in one static executable/mapping
profile. General process tables, preemptive user scheduling, pipes/other blocking
I/O, signal delivery, dynamic/general ELF, writable storage, networking/isolation,
ordinary general kernel libraries, distribution replacement, Docker/Podman and
VM-host acceptance remain. AArch64/RISC-V require their own native return/retry,
extended-state and mapping proofs; x86 instruction details remain private.

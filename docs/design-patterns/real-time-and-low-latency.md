# Real-time and low-latency patterns

## RT-01 — Time-triggered cyclic execution

**Good fit.** A closed set of periodic control, sensing, and communication
actions must have repeatable timing and analyzable interference.

**References.** Kopetz and Bauer describe the time-triggered architecture for
dependable distributed embedded systems with a precise global time base.[^rt1]

**Hardware assumptions.** Real-time clock/timer, bounded interrupt latency,
and usually MCU/DSP/real-time CPU; distributed use needs synchronized clocks.

**Problem.** Event-driven arrival order makes timing, integration, and fault
containable-causality hard to analyze.

**Structure.** Build a static repeating schedule of release, compute,
communication, and actuation slots against absolute time; isolate mode changes
as explicit schedules.

**Limitations.** It is inflexible, can waste capacity, and may delay sporadic
events. Clock drift, overload, and WCET errors invalidate timing arguments.

**Core-language support required.** Absolute/monotonic time, periodic releases,
static task sets, bounded execution and communication, schedule tables,
hardware clock/effect models, explicit mode transitions, and target-checked
deadline evidence.

## RT-02 — Rate-monotonic fixed-priority task set

**Good fit.** Independent periodic tasks have known periods, deadlines, and
worst-case execution times and run under preemptive fixed-priority scheduling.

**References.** Liu and Layland establish schedulability results for fixed- and
dynamic-priority hard-real-time task sets.[^rt2] Ravenscar adopts deterministic
fixed-priority tasking suitable for analysis.[^rt3]

**Hardware assumptions.** Real-time scheduler and timer; classic bounds assume
one processor and specific independence/deadline conditions.

**Problem.** Without analyzable priority assignment, lower-frequency work can
cause missed deadlines despite adequate average utilization.

**Structure.** Give shorter-period tasks higher static priority, release them
periodically using absolute time, and admit the task set only after response-
time or utilization analysis.

**Limitations.** Shared resources, jitter, multicore migration, non-periodic
work, and deadlines unequal to periods require stronger analysis.

**Core-language support required.** Period/deadline/WCET declarations, static
priority and CPU affinity, preemption semantics, absolute delay, bounded task
sets, interference/resource models, and compiler/linker schedulability checks.

## RT-03 — Priority-ceiling protected resource

**Good fit.** Fixed-priority real-time tasks must share short critical sections
with bounded priority inversion and no lock-order deadlock.

**References.** Sha et al. describe a priority-ceiling locking protocol with
deadlock freedom and bounded blocking under its assumptions.[^rt4] Ravenscar
standardizes ceiling locking in its restricted tasking profile.[^rt3]

**Hardware assumptions.** Preemptive priority scheduler and mutual exclusion;
classic guarantees are scheduler- and processor-model specific.

**Problem.** A low-priority lock holder can block high-priority work for an
unbounded time; nested locks can deadlock.

**Structure.** Give each protected resource the highest priority of any client;
enforce ceiling admission and raise the holder's active priority while locked.

**Limitations.** Requires complete client/priority knowledge, adds blocking,
and scales poorly to general multicore locking or dynamic task creation.

**Core-language support required.** Source-visible protected resources or locks,
static priorities/ceilings, critical-section bounds, nonblocking-in-section
rules, scheduler semantics, and compile-time ceiling/deadlock validation.

## RT-04 — Busy-poll run-to-completion loop

**Good fit.** A dedicated core processes a high-rate queue or device and lower
tail latency is worth consuming the core continuously.

**References.** DPDK documents interrupt-free polling and run-to-completion
packet loops.[^rt5] Linux NAPI explicitly describes the CPU-for-latency tradeoff
of busy polling.[^rt6]

**Hardware assumptions.** Dedicated CPU core, coherent descriptor rings,
stable frequency/topology, and usually direct device queues.

**Problem.** Interrupts, wakeups, migrations, allocation, and cross-core queues
introduce latency and jitter in a hot path.

**Structure.** Pin one loop to a core, poll a bounded batch, process and emit on
that core, preallocate state, and define an overload/shutdown path.

**Limitations.** Burns energy and capacity while idle; large batches hurt
latency and fairness; OS noise, SMT, NUMA, and thermal changes still add jitter.

**Core-language support required.** Explicit endless/productive loops, device
poll effects, fixed buffers, `NoAlloc`, batch bounds, affinity/isolation and
polling policies, target timing evidence, and cooperative shutdown semantics.

## RT-05 — Split interrupt top half and deferred work

**Good fit.** Hardware demands prompt acknowledgement but the full handler is
too long or effectful for interrupt context.

**References.** Zephyr recommends keeping an ISR short and offloading longer
processing to a thread or work queue.[^rt7]

**Hardware assumptions.** Interrupt controller, ISR context, and a safe queue
or wake mechanism to normal execution.

**Problem.** Long ISR work increases interrupt latency, jitter, and priority
inversion; unsafe calls in interrupt context can block or corrupt state.

**Structure.** The top half performs bounded capture/acknowledgement and posts
a compact work item; a typed lower-priority handler performs the rest.

**Limitations.** Deferral adds queue latency and overflow policy; shared state
crosses contexts; acknowledgement order is device-specific.

**Core-language support required.** An interrupt execution context with a
restricted effect/allocation set, volatile/MMIO ordering, bounded handoff,
ownership transfer, priority/latency declarations, and target-specific binding.

## RT-06 — Bounded-work / WCET-friendly control flow

**Good fit.** A deadline or resource budget needs a defensible upper bound over
every path rather than good average performance.

**References.** SPARK combines contracts and proof of run-time safety.[^rt8]
Ravenscar removes dynamic features to keep concurrency and sequential execution
predictable.[^rt3]

**Hardware assumptions.** A characterized target is required for time bounds;
the structural pattern also helps memory-constrained MCU/DSP code.

**Problem.** Unbounded recursion, data-dependent loops, allocation, dynamic
dispatch, caches, and retries obscure worst-case time and space.

**Structure.** Make loop/recursion bounds and sizes explicit; use finite state,
static dispatch, bounded queues, and no hot-path allocation; verify each call's
transitive bound.

**Limitations.** Conservative bounds waste capacity and may reject valid code;
modern caches, speculation, accelerators, and interrupts need platform models.

**Core-language support required.** Termination/decrease evidence, numeric and
collection bounds, transitive cost contracts including constant factors/cycles
and peak space, static call alternatives, `NoAlloc`, and target-specific WCET
verification.

## Sources

[^rt1]: H. Kopetz and G. Bauer, [“The Time-Triggered Architecture”](https://doi.org/10.1109/JPROC.2002.805821), *Proceedings of the IEEE*, 2003.
[^rt2]: C. Liu and J. Layland, [“Scheduling Algorithms for Multiprogramming in a Hard-Real-Time Environment”](https://doi.org/10.1145/321738.321743), *JACM*, 1973.
[^rt3]: Ada Reference Manual, [The Ravenscar Profile](https://docs.adacore.com/live/wave/arm12/html/arm12/arm12-D-13.html).
[^rt4]: L. Sha et al., [*A Real-Time Locking Protocol*](https://www.sei.cmu.edu/library/a-real-time-locking-protocol/), CMU/SEI-89-TR-018, 1989.
[^rt5]: DPDK, [Poll Mode Driver](https://doc.dpdk.org/guides/prog_guide/poll_mode_drv.html).
[^rt6]: Linux kernel, [NAPI — Busy Polling](https://docs.kernel.org/networking/napi.html#busy-polling).
[^rt7]: Zephyr Project, [Interrupts — Offloading ISR Work](https://docs.zephyrproject.org/latest/kernel/services/interrupts.html#offloading-isr-work).
[^rt8]: AdaCore, [SPARK Reference Manual introduction](https://docs.adacore.com/spark2014-docs/html/lrm/introduction.html).

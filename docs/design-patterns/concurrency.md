# Concurrency patterns

## CC-01 — Structured fork/join with work stealing

**Good fit.** A computation forms a well-nested dynamic DAG of independent
subcomputations whose load balance is not known statically.

**References.** Cilk work stealing gives bounds in terms of serial work and
critical-path length for fully strict computations.[^cc1]

**Hardware assumptions.** Multicore/shared-memory CPU; hierarchical variants
can span NUMA nodes.

**Problem.** Manually assigned threads load-balance poorly, oversubscribe, and
leak lifetimes; a global work queue contends.

**Structure.** Spawn child work inside a lexical scope, join before scope exit,
keep local work on per-worker deques, and let idle workers steal.

**Limitations.** Random stealing adds scheduling jitter and is unsuitable for
hard deadlines without a constrained scheduler. Blocking effects and fine tasks
destroy efficiency; space can grow with worker count.

**Core-language support required.** Structured child lifetimes, explicit
independence/effects, safe value sharing and transfer, join/cancellation,
work/span evidence, grain-size specialization, and runtime scheduler contracts.

## CC-02 — Actor / serial event loop

**Good fit.** Independently evolving components own private state and interact
asynchronously without shared mutable memory.

**References.** Hewitt, Bishop, and Steiger's Actor formalism models computation
through actors receiving messages and creating/sending to actors.[^cc2]

**Hardware assumptions.** None; actors can be local, multicore, or distributed.

**Problem.** Shared-state locking couples components and admits races,
deadlocks, and failure propagation.

**Structure.** Each actor owns state, processes one state-changing message at a
time, communicates through typed messages, and exposes capabilities rather
than state references.

**Limitations.** Mailbox growth, reentrancy/suspension, ordering between
senders, and distributed failure remain hard. Actor isolation alone gives no
determinism or backpressure.

**Core-language support required.** Isolated task state, typed asynchronous
send/request/stream forms, endpoint authority, defined mailbox admission and
ordering, ownership transfer, structured lifecycle, timeout, and supervision.

## CC-03 — CSP channel network with protocol types

**Good fit.** A system is naturally a composition of sequential processes with
explicit communication and synchronization topology.

**References.** Hoare presents input/output and parallel composition as program
structuring primitives.[^cc3] Session types add protocol-state checking.[^cc4]

**Hardware assumptions.** None; channels can lower to direct calls, queues,
shared memory, or transport.

**Problem.** Callback graphs hide communication order and make deadlock,
message mismatch, and incomplete protocol handling difficult to see.

**Structure.** Processes communicate only through typed channels; guarded
choice and protocol states specify which communication may occur next.

**Limitations.** Cyclic wait and starvation remain possible without analysis;
static topologies are less flexible; distributed channels add partial failure.

**Core-language support required.** Typed bounded channels, synchronous and
asynchronous interactions, linear/session endpoints, guarded choice, protocol
closure, dependency/deadlock analysis, and implementation-transparent lowering.

## CC-04 — Preallocated single-writer ring / Disruptor

**Good fit.** A fixed pipeline needs very high throughput and low jitter, and a
single writer (or separately sequenced writers) can publish ordered events.

**References.** LMAX documents a preallocated ring, single-writer principle,
sequence barriers, dependency graphs, and cache-conscious layout.[^cc5]

**Hardware assumptions.** Cache-coherent multicore CPU with memory-ordering
primitives; false-sharing behavior matters.

**Problem.** General queues combine storage, contention, allocation, and
wakeup policy, adding latency and jitter.

**Structure.** Reuse fixed ring slots; one writer claims and publishes sequence
numbers; consumers advance independent cursors; capacity prevents overwrite.

**Limitations.** Fixed capacity requires overload policy; busy waiting burns
cores; multi-producer sequencing and wraparound are subtle; slow consumers
stall producers.

**Core-language support required.** Fixed contiguous layout, cache-line
alignment/padding, wrapping counters, source-visible acquire/release atomics or
an intrinsic channel with equivalent guarantees, preallocation, affinity/wait
policy, and bounded backpressure.

## CC-05 — Read-copy-update with safe reclamation

**Good fit.** Read-mostly shared structures require extremely cheap readers
while updates and reclamation may be deferred.

**References.** Linux documents RCU as a synchronization alternative whose
updaters do not block readers.[^cc6] Hazard pointers provide portable safe
reclamation for lock-free objects.[^cc7]

**Hardware assumptions.** Shared-memory multiprocessor with atomics and a
specified memory model; RCU may depend on scheduler/quiescent-state support.

**Problem.** Reader-writer locks add cache traffic and blocking to common
reads; freeing replaced nodes while readers retain them causes use-after-free.

**Structure.** Publish a new immutable version atomically; readers use cheap
read-side protection; reclaim the old version only after every prior reader is
quiescent or no hazard pointer protects it.

**Limitations.** Reclamation may be delayed and memory can grow; memory-order
proofs are difficult; stalled participants can break bounded-space claims.

**Core-language support required.** Source-level atomic load/store/RMW and
orders, non-owning protected references, epochs or hazard identities,
quiescence/liveness assumptions, explicit reclamation, and unsafe/verified
boundary rules. A compiler-private implementation is insufficient when the
algorithm itself observes these operations.

## CC-06 — Composable software transaction

**Good fit.** Several shared-state operations must compose atomically and retry
on conflicts without exposing lock order.

**References.** Harris et al. present composable memory transactions with
modular blocking and choice.[^cc8]

**Hardware assumptions.** Shared-memory CPU; implementation may use software
logs, locks, or hardware transactional memory.

**Problem.** Correct lock-based components do not automatically compose, and
multi-lock operations introduce deadlock and partial update.

**Structure.** Execute a transaction speculatively, record reads/writes, commit
atomically if validation succeeds, otherwise retry; compose transactions as
values.

**Limitations.** Conflicts can starve, irrevocable I/O cannot be rolled back,
logging costs memory, and worst-case time is hard to bound.

**Core-language support required.** Transactional state/effect isolation,
retry-safe and irrevocable-effect classification, atomic commit semantics,
composable choice, exception/cancellation rollback, and progress guarantees or
explicitly stated assumptions.

## Sources

[^cc1]: R. Blumofe and C. Leiserson, [“Scheduling Multithreaded Computations by Work Stealing”](https://www.cs.cornell.edu/courses/cs612/2006sp/papers/blumofe94.pdf), FOCS, 1994.
[^cc2]: C. Hewitt, P. Bishop, and R. Steiger, [“A Universal Modular ACTOR Formalism for Artificial Intelligence”](https://www.eighty-twenty.org/files/Hewitt%2C%20Bishop%2C%20Steiger%20-%201973%20-%20A%20universal%20modular%20ACTOR%20formalism%20for%20artificial%20intelligence.pdf), IJCAI, 1973.
[^cc3]: C. A. R. Hoare, [“Communicating Sequential Processes”](https://doi.org/10.1145/359576.359585), *CACM*, 1978.
[^cc4]: PLS Lab, [Session Types](https://www.pls-lab.org/Session_Types), with the primary session-type bibliography.
[^cc5]: LMAX Exchange, [*Disruptor Technical Paper*](https://lmax-exchange.github.io/disruptor/disruptor.html).
[^cc6]: Linux kernel, [A Tour Through RCU's Requirements](https://docs.kernel.org/RCU/Design/Requirements/Requirements.html).
[^cc7]: M. Michael, [“Hazard Pointers: Safe Memory Reclamation for Lock-Free Objects”](https://research.ibm.com/publications/hazard-pointers-safe-memory-reclamation-for-lock-free-objects), *IEEE TPDS*, 2004.
[^cc8]: T. Harris, S. Marlow, S. Peyton Jones, and M. Herlihy, [“Composable Memory Transactions”](https://cs.brown.edu/people/mph/HarrisMPJH05/stm.pdf), PPoPP, 2005.

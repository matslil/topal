# Loop and memory optimizations

## CO-LM-001 — Loop-invariant code motion and guarded versioning

**Found in.** LLVM's production loop-invariant code-motion (`licm`) pass hoists
or sinks invariant operations and may promote must-alias memory to scalars; its
documentation identifies alias and memory-effect analysis as prerequisites.[^lm1]

**Problem.** A loop repeatedly executes an operation or memory access whose
value does not change between iterations.

**Approach.** Move safe invariant work to a preheader or exits. When facts are
uncertain but cheaply testable, create a guarded optimized loop and a general
fallback.

**Assumptions and evidence.** Hoisting requires dominance, loop invariance,
alias and effect facts, and proof that executing earlier does not expose a
failure or nontermination on paths that previously skipped the operation.
Versioning requires a complete runtime predicate and correct fallback.

**Limitations and interactions.** Hoisting extends live ranges and can increase
register pressure. Versioning adds branches and code size. Moving a may-fail or
resource-consuming Topal operation can change observations even if its value is
invariant.

**Characteristic weighting.** Weigh saved work times expected trip count
against register spills, guard cost, code growth, cache pressure, and cold-loop
startup. Profiled trip counts improve the estimate.

**Topal relevance.** Topal must establish effect, failure, and cleanup safety;
LLVM can perform low-level LICM when supplied accurate alias and memory-effect
information.

## CO-LM-002 — Unrolling, peeling, and loop unswitching

**Found in.** LLVM documents loop unrolling, peeling, unroll-and-jam, and
unswitching; its unswitch pass explicitly limits transformation because each
invariant condition can duplicate a loop.[^lm1]

**Problem.** Loop control, invariant branches, short dependency windows, or
unknown boundary iterations prevent efficient straight-line and vector code.

**Approach.** Duplicate several iterations, separate leading or trailing
iterations, fuse unrolled inner bodies, or clone the loop around an invariant
condition.

**Assumptions and evidence.** The compiler needs valid trip-count or residual
logic, dependence and effect safety, code-size estimates, branch probabilities,
and target information about pipelines, vectors, registers, and instruction
cache.

**Limitations and interactions.** Code growth can overwhelm instruction cache,
registers, compilation, and embedded storage. Unknown or tiny trip counts may
not amortize guards. Unrolling may enable vectorization and scalar replacement
or cause spills that erase the benefit.

**Characteristic weighting.** Speed profiles accept growth up to a target- and
hotness-dependent threshold; size profiles generally restrict it. The chosen
factor balances branch overhead and instruction-level parallelism against
register pressure and code size.

**Topal relevance.** Preserve exact repetition semantics and effect order in
Topal IR; leave ordinary unroll factors to LLVM unless a Topal bulk operation
needs semantic restructuring first.

## CO-LM-003 — Loop and superword-level vectorization

**Found in.** LLVM has production loop and SLP vectorizers. The loop vectorizer
widens iterations, SLP combines isomorphic scalar statements, and a target cost
model selects vector and interleave factors.[^lm2] GCC enables corresponding
tree loop and SLP vectorizers in optimized pipelines.[^lm3]

**Problem.** Scalar code underuses SIMD or scalable-vector execution resources.

**Approach.** Prove iterations or scalar lanes independent, widen operations
and memory accesses, generate masks or a scalar remainder, and select fixed or
scalable vector shapes supported by the target.

**Assumptions and evidence.** Requires dependence, alias, alignment, stride,
trip-count, effect, failure, and numerical-reassociation facts. The target model
must describe legal vector types, masked operations, gather/scatter, conversion,
and throughput costs.

**Limitations and interactions.** Guards, masks, shuffles, gathers, reductions,
and remainders can cost more than scalar execution. Wider vectors increase
register pressure and may reduce clock frequency or portability. Exact or
order-sensitive arithmetic may forbid common reductions.

**Characteristic weighting.** Compare expected lane utilization and throughput
with setup, scalar cleanup, memory bandwidth, register pressure, code size,
energy, and small-input latency. Size-oriented profiles may prefer one compact
scalar loop.

**Topal relevance.** Topal can provide independence, shapes, ranges, alignment,
and legal reduction evidence. LLVM should receive vectorizable IR and choose
machine vector widths; accelerator-specific bulk lowering may remain Topal-owned.

## CO-LM-004 — Loop fusion and distribution

**Found in.** LLVM documents a production loop-fusion pass.[^lm1] Polly performs
fusion as a polyhedral locality transformation.[^lm4] MLIR Linalg supports
tiled producer-consumer fusion on structured operations.[^lm5]

**Problem.** Separate loops may reread the same data or materialize an
intermediate, while one large loop may have excessive live state, poor
parallelism, or incompatible access patterns.

**Approach.** Fuse compatible producer and consumer iteration spaces to reuse
values, or distribute statements into separate loops to reduce dependencies and
enable vectorization, parallelism, or different schedules.

**Assumptions and evidence.** Requires exact dependence, iteration-domain,
effect-order, alias, failure, and shape information. Fusion needs compatible
execution spaces; distribution must preserve cross-statement dependences.

**Limitations and interactions.** Fusion can increase registers, inhibit
parallelism, duplicate computation, or combine incompatible locality patterns.
Distribution adds loop overhead and memory traffic and can force materialization.

**Characteristic weighting.** Balance eliminated transfers and better locality
against live-state pressure, occupancy, vectorizability, parallel span, code
size, and recomputation. CPU cache and accelerator HBM/register trade-offs differ.

**Topal relevance.** This is a likely Topal implementation-plan decision because
it can erase semantic intermediates only with effect and allocation evidence;
structured output should still expose resulting loops to LLVM.

## CO-LM-005 — Interchange, tiling, and polyhedral scheduling

**Found in.** Polly represents affine iteration and access sets with integer
polyhedra and applies tiling and related loop transformations for locality and
parallelism.[^lm4] MLIR Linalg makes parametric tiling, fast-memory promotion,
and hardware mapping central transformations.[^lm5]

**Problem.** A legal iteration order may traverse memory poorly, overflow a
fast-memory level, or conceal parallel dimensions.

**Approach.** Reorder loop dimensions and partition iteration domains into
tiles sized for cache, scratchpad, vectors, threads, or accelerator units. A
polyhedral scheduler searches legal affine schedules under dependences.

**Assumptions and evidence.** Needs analyzable bounds and affine or otherwise
structured accesses, dependence information, shapes, strides, memory hierarchy,
parallel resources, and boundary semantics.

**Limitations and interactions.** Irregular access, data-dependent control,
opaque calls, and complex effects fall outside simple affine models. One tile
size rarely suits every cache level, shape, or concurrent workload. Boundary
code and search time can be substantial.

**Characteristic weighting.** Optimize reuse, transfer volume, parallelism,
vector fit, and occupancy subject to fast-memory capacity, alignment, bank
conflicts, code size, compile time, and small-problem overhead. Often this is a
multi-level or autotuned choice rather than one scalar score.

**Topal relevance.** Topal's typed implementation-plan IR already anticipates
tiling and iteration choices. The architecture model must supply memory and
execution facts while legality comes from shapes, effects, and dependencies.
See also design patterns [MM-06](../design-patterns/memory-and-cpu.md) and
[HA-02](../design-patterns/accelerators-and-dsp.md).

## CO-LM-006 — Prefetching and latency hiding

**Found in.** GCC exposes loop-array prefetching among its production
optimization options.[^lm3] TVM describes memory-latency hiding as an
architecture-specific scheduling problem for tensor programs.[^lm6]

**Problem.** Computation stalls while waiting for predictable future memory or
device transfers.

**Approach.** Issue nonbinding cache prefetches, software-pipeline loads, or
asynchronous copies sufficiently ahead of use; overlap stages and retain a
correct synchronous fallback.

**Assumptions and evidence.** Requires predictable addresses, enough independent
work, safe address formation, cache or DMA semantics, transfer latency and
bandwidth, queue limits, ownership, completion, and barrier information.

**Limitations and interactions.** Early fetches consume bandwidth, cache,
queues, and energy; they may evict useful data or cross invalid pages if address
formation is not safe. Timing varies under contention. Too much distance
increases live state.

**Characteristic weighting.** Trade covered latency against bandwidth, cache
pollution, energy, register/buffer usage, queue pressure, and predictability.
Hard real-time analysis needs bounded completion evidence, not average benefit.

**Topal relevance.** Ordinary CPU prefetch selection can be delegated to LLVM.
DMA or asynchronous staging is a Topal plan decision requiring explicit memory
domains, ownership, completion, and barriers. See [HA-05](../design-patterns/accelerators-and-dsp.md).

## CO-LM-007 — Physical layout selection and propagation

**Found in.** XLA separates logical tensor shape from physical layout, assigns
operation-preferred layouts, propagates them through a graph, and materializes
copies at conflicts.[^lm7] MLIR Linalg likewise preserves structured iteration
and layout information for lowering.[^lm5]

**Problem.** A semantic field or dimension order can cause strided loads,
gathers, bank conflicts, poor cache-line use, or incompatibility with a target
operator.

**Approach.** Select dimension order, blocking, structure-of-arrays, packing,
alignment, and memory space independently of semantic value shape; propagate a
compatible choice and insert conversions only at unavoidable boundaries.

**Assumptions and evidence.** Needs complete use graph or boundary contracts,
shapes and access frequencies, ABI/serialization separation, target transaction
and operator requirements, lifetime, and conversion costs.

**Limitations and interactions.** Consumers can prefer conflicting layouts.
Conversions add time and peak memory; target-private layouts hurt
interoperability and debugging. Dynamic shapes make static choices uncertain.

**Characteristic weighting.** Balance per-use locality and vector/operator
efficiency against conversion count, peak storage, padding, code complexity,
external layout obligations, and reuse across separately compiled code.

**Topal relevance.** Representation selection is Topal-owned because semantic,
foreign, persistent, and debug layouts must remain distinct. LLVM should see
the selected explicit layout. See [MM-05](../design-patterns/memory-and-cpu.md)
and [HA-03](../design-patterns/accelerators-and-dsp.md).

## CO-LM-008 — Bufferization, reuse, and memory planning

**Found in.** MLIR One-Shot Bufferize converts tensor values to memory
references late so tensor-level fusion and tiling remain available.[^lm8] XLA
uses whole-graph scheduling and buffer assignment to minimize intermediate
memory.[^lm7]

**Problem.** A direct lowering of immutable aggregates can allocate and copy an
intermediate for every value, increasing peak memory and traffic.

**Approach.** Determine lifetimes and aliasing, choose in-place destinations,
reuse nonoverlapping buffers, coalesce storage, and schedule operations to
reduce simultaneous live memory while preserving value semantics.

**Assumptions and evidence.** Requires exact shapes and sizes or safe bounds,
ownership, exclusivity, liveness, alias and escape facts, memory-space
compatibility, alignment, failure cleanup, and whether allocation is observable.

**Limitations and interactions.** Reuse can constrain scheduling and parallelism,
retain oversized buffers, expose old sensitive contents, or introduce copies
when guesses are wrong. Peak-memory minimization may lengthen execution.

**Characteristic weighting.** Balance allocation count, copy traffic, peak and
retained memory, parallel span, locality, fragmentation, cleanup cost, and
predictability. A memory cap is a hard constraint rather than a soft weight.

**Topal relevance.** Topal's immutable semantics, exclusivity evidence, regions,
and resource bounds make this a Topal-owned plan transformation. Selected
buffers and lifetimes should lower explicitly to LLVM. See
[MM-01](../design-patterns/memory-and-cpu.md) and
[MM-02](../design-patterns/memory-and-cpu.md).

## Sources

[^lm1]: LLVM Project, [LLVM's Analysis and Transform Passes](https://llvm.org/docs/Passes.html).
[^lm2]: LLVM Project, [Auto-Vectorization in LLVM](https://llvm.org/docs/Vectorizers.html).
[^lm3]: GNU Project, [Optimize Options](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html).
[^lm4]: LLVM Polly, [LLVM Framework for High-Level Loop and Data-Locality Optimizations](https://polly.llvm.org/).
[^lm5]: LLVM Project, [MLIR Linalg Dialect](https://mlir.llvm.org/docs/Dialects/Linalg/).
[^lm6]: T. Chen et al., [“TVM: An Automated End-to-End Optimizing Compiler for Deep Learning”](https://www.usenix.org/conference/osdi18/presentation/chen), OSDI, 2018.
[^lm7]: OpenXLA Project, [XLA:GPU Architecture Overview](https://openxla.org/xla/gpu_architecture).
[^lm8]: LLVM Project, [MLIR Bufferization](https://mlir.llvm.org/docs/Bufferization/).

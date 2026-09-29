# Search, learning, runtime, and post-link optimizations

## CO-SP-001 — Equality saturation and cost-based extraction

**Found in.** The `egg` research system uses e-graphs to represent many
equivalent expressions simultaneously, applies rewrite rules without choosing
an order immediately, and extracts a result with a cost function.[^sp1]
Cranelift uses a bounded acyclic e-graph design in a production-oriented
code generator to retain alternatives without unrestricted saturation.[^sp10]

**Problem.** Greedy local rewriting suffers from phase ordering: choosing one
equivalent form can hide another rewrite and lead to a poor local minimum.

**Approach.** Add equivalent expressions to an e-graph until a resource or
saturation limit, run domain analyses over equivalence classes, then extract a
legal expression minimizing a target-dependent cost.

**Assumptions and evidence.** Each equality must be valid under exact language
semantics and side conditions. Extraction needs an acyclic realizable result
and a cost model that accounts for sharing, effects, target operations, and
possibly multiple objectives.

**Limitations and interactions.** E-graphs can grow explosively; unrestricted
rewrites may never saturate. Simple additive extraction costs poorly represent
register pressure, scheduling, shared subexpressions, caches, or global layout.
Effects and control flow need richer representations than pure terms.

**Characteristic weighting.** The extraction cost can combine latency, size,
energy, and target legality, but hard semantic and resource constraints must be
applied before ranking. Compilation memory and time require explicit fuel and
growth limits.

**Topal relevance.** Equality saturation could explore law-based pure Topal
expressions or schedule fragments while preserving proof provenance. It should
not replace effect, failure, ownership, or evidence checking.

## CO-SP-002 — Solver-backed superoptimization

**Found in.** Souper synthesizes replacements for LLVM integer expressions and
uses solver reasoning to establish them.[^sp2] STOKE searches x86-64 binary
sequences stochastically and combines correctness and performance terms in its
search cost.[^sp3]

**Problem.** Handwritten peephole rules cover only a small fraction of profitable
instruction sequences and are difficult to tune across targets.

**Approach.** Enumerate, synthesize, or stochastically mutate candidate programs;
reject or penalize incorrect candidates using tests and formal solvers; rank
survivors by measured or modeled target cost.

**Assumptions and evidence.** Needs a complete semantics for the searched IR or
ISA subset, bounded live inputs/outputs and memory, solver-valid equivalence,
target feature identity, and a reliable cost measurement or model.

**Limitations and interactions.** Search and proof can be extremely expensive;
timeouts are normal and completeness is limited. Undefined behavior or
incomplete instruction semantics can produce unsound rewrites. Results may
overfit one microarchitecture and be hard to debug.

**Characteristic weighting.** Correctness is a hard gate. Search then balances
measured latency or throughput with code size and sometimes energy; compilation
budget bounds the search. Offline-discovered, validated rewrites amortize cost
better than per-build search.

**Topal relevance.** Use only over a precisely defined lowered subset, with
proof certificates or independent validation and exact architecture identity.
LLVM already owns most machine peepholes; Topal-specific exact arithmetic or
semantic operations could justify a narrower synthesizer.

## CO-SP-003 — Schedule autotuning and empirical search

**Found in.** Halide separates an image-processing algorithm from schedules
covering storage, computation, vector, and parallel choices.[^sp4] TVM searches
schedule spaces and measures generated candidates on target hardware.[^sp5]

**Problem.** Interacting choices such as tile sizes, fusion, layout, vector
width, and launch geometry produce a large architecture- and workload-dependent
space that static heuristics model imperfectly.

**Approach.** Define a legal parameterized schedule space, prune it with static
constraints or learned models, compile candidates, measure representative
workloads on the target, and retain the best validated plan.

**Assumptions and evidence.** Needs semantics-preserving schedule operations,
target access, stable measurement, representative shapes and data, bounded
search, reproducible environment identity, and a portable correct fallback.

**Limitations and interactions.** Tuning is expensive, noisy, and prone to
overfitting. It may be impossible during cross-compilation or deployment. New
hardware, toolchains, concurrent load, or input distributions invalidate
results.

**Characteristic weighting.** Empirical objectives may combine median or tail
time, throughput, energy, peak memory, and code size subject to correctness and
resource caps. The tuning budget and number of deployed variants are explicit
costs.

**Topal relevance.** A typed implementation-plan search can tune legal Topal
schedule alternatives. Measurements are evidence tied to exact model, board,
toolchain, and workload identities, not timeless architecture facts. See
[HA-06](../design-patterns/accelerators-and-dsp.md).

## CO-SP-004 — Machine-learned optimization heuristics

**Found in.** LLVM MLGO integrates learned policies for inlining-for-size and
register-allocation eviction while retaining the surrounding production pass
and corpus tooling.[^sp6] The MLGO paper reports training an inlining policy
against object-size outcomes.[^sp7]

**Problem.** Hand-tuned heuristics for combinatorial decisions are difficult to
maintain and may generalize poorly across programs and architectures.

**Approach.** Extract compiler-state features, train a model on a defined corpus
and reward, embed a compatible inference model, and let it advise a bounded
decision point with a conventional fallback.

**Assumptions and evidence.** Requires reproducible corpora and labels, stable
features, compatible compiler/model revisions, bounded deterministic inference,
validation on held-out workloads and targets, and preservation of pass legality
independent of the model.

**Limitations and interactions.** Models inherit corpus bias, drift as the
compiler changes, are difficult to explain, and add build/training complexity.
An average reward may hide severe regressions. Learned output cannot establish
semantic correctness.

**Characteristic weighting.** The training reward makes weighting explicit:
for example size, speed, or spill cost. Deployment must additionally bound
inference time, model bytes, compile-time variance, worst regressions, and
fallback behavior.

**Topal relevance.** A learned model may rank already legal alternatives but
must carry version, corpus, architecture, objective, and validation identity.
Deterministic rule-based policy should remain available for reproducible and
high-assurance builds.

## CO-SP-005 — Tiered compilation and speculative optimization

**Found in.** HotSpot combines interpretation, faster profiling compilation,
and a more optimizing server compiler; its tiered system uses execution feedback
to improve startup and later peak performance.[^sp8]

**Problem.** Expensive optimization delays startup, while compiling everything
quickly leaves hot long-running code slow and lacks runtime type/value facts.

**Approach.** Start with interpretation or cheap code, collect counters and type
profiles, optimize hot regions under guarded assumptions, and deoptimize to a
semantically complete state if an assumption fails.

**Assumptions and evidence.** Requires a managed runtime, patchable call paths,
precise state maps, safe points, deoptimization semantics, profile storage,
code-cache management, and bounded interaction with effects and external state.

**Limitations and interactions.** Runtime, memory, latency, and implementation
complexity are high. Warmup and recompilation make timing less predictable;
deoptimization metadata consumes space. This is unsuitable for many freestanding
or hard-real-time deployments.

**Characteristic weighting.** Balance startup, warmup, peak throughput,
latency variance, compilation CPU, code-cache memory, energy, and deployment
lifetime. Thresholds should reflect expected reuse rather than only peak speed.

**Topal relevance.** This is not part of the current AOT freestanding compiler.
It remains a future runtime strategy; AOT multiversioning is distinct because
its alternatives and dispatch are fixed in the artifact.

## CO-SP-006 — Profile-guided post-link binary optimization

**Found in.** BOLT is a production-oriented LLVM post-link optimizer that uses
profiles and final binary information to reorder code and apply binary-level
optimizations after conventional compilation and linking.[^sp9]

**Problem.** Earlier stages estimate final addresses, padding, branch ranges,
and whole-program hot layout; large binaries retain instruction-cache, TLB, and
branch costs even after PGO and LTO.

**Approach.** Decode the linked binary with relocations and profile mappings,
reconstruct control flow, reorder functions and blocks, split hot/cold code,
relax branches, and emit updated metadata.

**Assumptions and evidence.** Requires a supported object/ISA format, complete
relocation and symbol information, trustworthy disassembly boundaries, a
matching execution profile, and preservation or regeneration of unwind, debug,
and address metadata.

**Limitations and interactions.** Stripped, obfuscated, hand-written, self-
modifying, or address-sensitive code can be unsafe to rewrite. Signing and
reproducibility occur after transformation. Profile drift and binary changes
reduce benefit.

**Characteristic weighting.** Favor hot instruction working-set density and
branch fall-through while bounding binary growth, padding, cold-start pages,
rewrite time, metadata quality, and profile mismatch risk.

**Topal relevance.** A later Topal native packaging stage could invoke a
qualified post-link optimizer, but it must preserve freestanding properties,
artifact digests, source mappings, signatures, and exact target/tool identities.

## Sources

[^sp1]: M. Willsey et al., [“egg: Fast and Extensible Equality Saturation”](https://doi.org/10.1145/3434304), POPL, 2021.
[^sp2]: N. Lopes et al., [“Souper: A Synthesizing Superoptimizer”](https://arxiv.org/abs/1711.04422), 2017.
[^sp3]: E. Schkufza, R. Sharma, and A. Aiken, [“Stochastic Superoptimization”](https://theory.stanford.edu/~aiken/publications/papers/asplos13.pdf), ASPLOS, 2013.
[^sp4]: J. Ragan-Kelley et al., [“Halide: A Language and Compiler for Optimizing Parallelism, Locality, and Recomputation in Image Processing Pipelines”](https://people.csail.mit.edu/jrk/halide-pldi13.pdf), PLDI, 2013.
[^sp5]: T. Chen et al., [“TVM: An Automated End-to-End Optimizing Compiler for Deep Learning”](https://www.usenix.org/conference/osdi18/presentation/chen), OSDI, 2018.
[^sp6]: LLVM Project, [Machine Learning-Guided Optimization](https://llvm.org/docs/MLGO.html).
[^sp7]: M. Trofin et al., [“MLGO: a Machine Learning Guided Compiler Optimizations Framework”](https://research.google/pubs/mlgo-a-machine-learning-guided-compiler-optimizations-framework/), 2021.
[^sp8]: Oracle, [Java HotSpot Virtual Machine Performance Enhancements](https://docs.oracle.com/en/java/javase/17/vm/java-hotspot-virtual-machine-performance-enhancements.html).
[^sp9]: M. Panchenko et al., [“BOLT: A Practical Binary Optimizer for Data Centers and Beyond”](https://arxiv.org/abs/1807.06735), CGO, 2019.
[^sp10]: Bytecode Alliance, [Cranelift e-graph optimizer RFC](https://github.com/bytecodealliance/rfcs/blob/main/accepted/cranelift-egraph.md).

# Target and heterogeneous lowering optimizations

## CO-TH-001 — Target-aware instruction selection and idiom recognition

**Found in.** LLVM's production code generator selects target instructions from
LLVM IR through SelectionDAG or GlobalISel and uses target descriptions for
legalization and pattern matching.[^th1] GCC machine descriptions similarly
describe target instruction patterns and costs.[^th2]

**Problem.** A target-independent operation or group of operations may have
many legal machine realizations with different instruction counts, latencies,
throughput, flags, addressing modes, and feature requirements.

**Approach.** Legalize unsupported types and operations, match IR graphs to
instruction patterns, and select a low-cost covering, including fused or
special instructions when their exact semantics match.

**Assumptions and evidence.** Requires exact target ISA, enabled features, ABI,
data layout, operation semantics, and a cost model. Pattern legality must cover
flags, trapping, rounding, saturation, memory ordering, and register constraints.

**Limitations and interactions.** Locally cheapest instructions may create
global register or scheduling problems. Incomplete patterns fall back to longer
sequences or helpers. A microarchitecture-tuned instruction may be unavailable
on the generic architecture baseline.

**Characteristic weighting.** Balance latency, reciprocal throughput,
micro-operations, code bytes, registers, energy, and feature portability.
Speed, size, and generic-compatibility profiles select different coverings.

**Topal relevance.** Topal should emit semantically precise, idiomatic LLVM IR
and target facts, leaving ordinary instruction selection to LLVM. Topal-owned
selection remains appropriate for semantic operations or accelerator choices
not faithfully represented by generic LLVM IR.

## CO-TH-002 — Instruction scheduling and software pipelining

**Found in.** LLVM orders selected instructions using target pipeline and
register-pressure information.[^th1] GCC processor descriptions model
functional-unit reservations for its scheduling hazard recognizer.[^th3]

**Problem.** A legal instruction order can serialize independent work, expose
long latencies, overuse one execution resource, or require too many live
registers.

**Approach.** Reorder instructions within dependence and barrier constraints,
interleave loop iterations, and initiate operations early enough to cover
latency while respecting machine pipelines.

**Assumptions and evidence.** Needs true and memory dependences, exception and
effect constraints, instruction latency/throughput, issue width, functional
units, hazards, register pressure, and in-order/out-of-order behavior.

**Limitations and interactions.** Scheduling models approximate dynamic
hardware and contention. Latency-oriented schedules may spill; pressure-
oriented schedules may expose stalls. Software pipelining adds prologue,
epilogue, and code-size cost and complicates precise failures.

**Characteristic weighting.** Trade critical-path latency and resource
utilization against registers, spills, code size, startup/drain overhead,
energy, and worst-case predictability. In-order DSPs and wide out-of-order CPUs
need different policies.

**Topal relevance.** LLVM should own machine scheduling. The architecture model
may supplement or validate LLVM scheduling data and guide higher-level plans,
but should not duplicate a backend scheduler without a demonstrated gap.

## CO-TH-003 — Register allocation, spilling, and rematerialization

**Found in.** LLVM's code generator maps unbounded SSA virtual registers to
target register classes, inserting spills as necessary.[^th1] LLVM MLGO also
supports a learned greedy-allocator eviction advisor.[^th4]

**Problem.** Physical registers are finite, classed, aliased, and constrained;
poor assignment causes moves, stack traffic, bank conflicts, or lost vector
occupancy.

**Approach.** Assign interfering live ranges to compatible physical registers,
split ranges, coalesce moves, spill selected values, or recompute cheap values
instead of loading them.

**Assumptions and evidence.** Requires liveness, register classes and aliases,
calling convention, instruction constraints, spill/load and rematerialization
costs, block frequency, stack alignment, and device occupancy effects.

**Limitations and interactions.** Optimal coloring is expensive; heuristics can
make locally irreversible choices. Aggressive inlining, unrolling, fusion, and
vectorization increase pressure. Spill costs depend on cache and stack placement.

**Characteristic weighting.** Minimize weighted spill and move cost while
respecting hard constraints; also consider code size, compile time, caller-save
traffic, vector occupancy, and critical paths. A size profile may accept slower
allocations or spills to avoid duplicated code.

**Topal relevance.** Leave allocation to LLVM. Topal's higher-level cost model
should predict pressure when comparing transformations and use emitted LLVM
shapes that do not unnecessarily prolong values.

## CO-TH-004 — Basic-block, function, and hot/cold layout

**Found in.** GCC optimized pipelines reorder blocks and functions and can
partition hot and cold code.[^th5] BOLT uses execution profiles after linking
to improve code layout for instruction caches and branch prediction.[^th6]

**Problem.** Semantically unrelated physical order causes taken branches,
instruction-cache and TLB misses, poor prefetch, and hot code diluted by cold
error paths.

**Approach.** Arrange likely successor blocks contiguously, form hot traces,
split cold fragments, and order functions using call or sample profiles after
actual sizes and addresses are known.

**Assumptions and evidence.** Needs branch/call frequencies or defensible
static estimates, final or estimated sizes, target branch ranges, cache/TLB
geometry, alignment cost, and exception/debug/unwind constraints.

**Limitations and interactions.** Layout overfits profiles, padding increases
binary size, and one workload's locality can harm another. Linker relaxation,
instrumentation, and address-sensitive interfaces can invalidate estimates.

**Characteristic weighting.** Favor hot fall-through and working-set density
against padding, branch range, binary size, page footprint, startup locality,
profile confidence, and reproducibility.

**Topal relevance.** LLVM and a future post-link stage should own physical
layout. Topal can identify semantic cold failure paths and preserve profile and
source mappings without fixing addresses prematurely.

## CO-TH-005 — Library, intrinsic, and accelerator-operator selection

**Found in.** MLIR Linalg can lower structured operations to libraries,
intrinsics, special instructions, or loops.[^th7] XLA selects between generated
GPU kernels and tuned libraries such as cuBLAS or cuDNN, noting that libraries
can preclude fusion.[^th8]

**Problem.** A compiler-generated general implementation may be slower than a
qualified vendor or platform routine, while a library call may add boundary
overhead and block cross-operation optimization.

**Approach.** Recognize a semantic operation or graph, verify exact compatibility,
and choose a target instruction, generated kernel, or checked library adapter;
retain a portable implementation as fallback.

**Assumptions and evidence.** Requires semantic equivalence, supported shapes,
layouts, precisions, alignment, error and rounding behavior, ABI, target/device
identity, library version, effects, workspace, and initialization costs.

**Limitations and interactions.** Libraries are opaque to fusion and may require
layout conversions, dynamic loading, large workspaces, or foreign runtimes.
Small inputs may not amortize dispatch. Vendor behavior and availability vary.

**Characteristic weighting.** Compare measured or modeled kernel time,
transfers, conversions, launch/setup, workspace and code size, energy,
qualification confidence, and portability. Unsupported semantics are a hard
rejection rather than a penalty.

**Topal relevance.** Selection belongs in the typed Topal implementation plan;
foreign or device operations need explicit checked adapters and must not
introduce an implicit runtime dependency.

## CO-TH-006 — Accelerator mapping and launch-geometry selection

**Found in.** MLIR Linalg maps structured parallel and reduction loops to
hardware, while the MLIR GPU dialect represents kernels, grids, blocks, threads,
and distinct GPU address spaces.[^th7][^th9] TVM searches tensor schedules for
diverse backends.[^th10]

**Problem.** A parallel computation must be partitioned across cores, warps,
vector lanes, tensor units, or DSP units with target-specific granularity and
resource limits.

**Approach.** Map iteration dimensions to hardware hierarchies, select grid,
block, warp and vector shapes, split reductions, and generate legal boundary
masks or residual work.

**Assumptions and evidence.** Needs independence/reduction laws, shapes,
divisibility, supported operations, synchronization, execution widths, maximum
threads, registers, local memory, occupancy rules, and launch overhead.

**Limitations and interactions.** Irregular work causes divergence and
imbalance. More threads can reduce per-thread locality or exceed registers and
local memory. Static shapes may not represent deployed dynamic workloads.

**Characteristic weighting.** Balance parallel occupancy, vector/tensor-unit
utilization, divergence, reduction cost, registers, local memory, launch
latency, transfers, energy, and small-input fallback. Several resource limits
are hard feasibility constraints.

**Topal relevance.** This is a Topal plan choice backed by architecture and
shape evidence, followed by target-specific lowering. See design pattern
[HA-01](../design-patterns/accelerators-and-dsp.md).

## CO-TH-007 — Memory-space placement and asynchronous staging

**Found in.** MLIR Linalg supports promotion to temporary fast memory; the GPU
dialect distinguishes global, workgroup, private, and constant spaces.[^th7][^th9]
XLA fusion and buffer assignment keep intermediates in registers/shared memory
and minimize HBM materialization.[^th8]

**Problem.** Uniform placement in a large slow memory wastes bandwidth and
latency when bounded reused data could reside in registers, scratchpad, SRAM,
cache, or another heterogeneous memory.

**Approach.** Assign values or tiles to memory spaces, insert necessary copies,
double-buffer or asynchronously stage transfers, and synchronize ownership and
visibility before use.

**Assumptions and evidence.** Requires capacity, bank and alignment rules,
addressability, transfer engines, latency/bandwidth, coherence, barriers,
lifetime, aliasing, ownership, completion, and fallback behavior.

**Limitations and interactions.** Fast memory is scarce; placement can reduce
occupancy or require expensive conversions. Noncoherent DMA and distributed
memories need explicit flush, invalidate, and failure handling. Static costs
may not capture contention.

**Characteristic weighting.** Minimize weighted transfer volume and access
latency subject to capacity and correctness, while considering overlap,
occupancy, peak memory, bank conflicts, energy, synchronization, and schedule
freedom.

**Topal relevance.** Topal must own semantic memory-domain, lifetime, transfer,
and barrier decisions. The architecture/board model supplies available spaces
and paths; LLVM or device backends lower the selected operations. See
[HA-05](../design-patterns/accelerators-and-dsp.md).

## CO-TH-008 — Function multiversioning and runtime feature dispatch

**Found in.** GCC and Clang support function multiversioning: they compile
clones for different architecture extensions and generate a resolver which
selects a compatible version at runtime, with target-specific mechanisms and
requirements.[^th11][^th12]

**Problem.** A binary intended for a broad architecture baseline cannot
otherwise use faster instructions present on only some deployment processors.

**Approach.** Compile selected functions under several compatible feature sets,
include a baseline fallback, detect execution-host features through a qualified
platform mechanism, and dispatch once or at a controlled call boundary.

**Assumptions and evidence.** Every clone must preserve identical Topal
semantics and calling contracts. The resolver needs trustworthy feature
detection, monotonic feature implications, target/OS support, initialization
ordering, thread safety, and a fallback runnable on the artifact baseline.

**Limitations and interactions.** Clones increase code and build time, inhibit
some cross-version sharing, and add resolver or indirect-call cost. Reported
features may not describe heterogeneous cores, emulation, migration, firmware
policy, or performance. Dynamic-linker resolvers do not fit every freestanding
platform.

**Characteristic weighting.** Compare expected speed or energy gain and
deployment coverage with clone bytes, instruction-cache pressure, detection and
dispatch overhead, startup, build time, artifact complexity, and confidence in
runtime facts. Variant count needs an explicit budget.

**Topal relevance.** This is distinct from explicit cross-compilation and
compile-time `native` tuning. A future Topal implementation must define its own
freestanding detection and dispatch boundary rather than silently acquiring a C
runtime or dynamic loader.

## Sources

[^th1]: LLVM Project, [The LLVM Target-Independent Code Generator](https://llvm.org/docs/CodeGenerator.html).
[^th2]: GNU Project, [Machine Descriptions](https://gcc.gnu.org/onlinedocs/gccint/Machine-Desc.html).
[^th3]: GNU Project, [Processor Pipeline Description](https://gcc.gnu.org/onlinedocs/gccint/Processor-pipeline-description.html).
[^th4]: LLVM Project, [Machine Learning-Guided Optimization](https://llvm.org/docs/MLGO.html).
[^th5]: GNU Project, [Optimize Options](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html).
[^th6]: Maksim Panchenko et al., [“BOLT: A Practical Binary Optimizer for Data Centers and Beyond”](https://arxiv.org/abs/1807.06735), CGO, 2019.
[^th7]: LLVM Project, [MLIR Linalg Dialect](https://mlir.llvm.org/docs/Dialects/Linalg/).
[^th8]: OpenXLA Project, [XLA:GPU Architecture Overview](https://openxla.org/xla/gpu_architecture).
[^th9]: LLVM Project, [MLIR GPU Dialect](https://mlir.llvm.org/docs/Dialects/GPU/).
[^th10]: T. Chen et al., [“TVM: An Automated End-to-End Optimizing Compiler for Deep Learning”](https://www.usenix.org/conference/osdi18/presentation/chen), OSDI, 2018.
[^th11]: GNU Project, [Function Multiversioning](https://gcc.gnu.org/onlinedocs/gcc/Function-Multiversioning.html).
[^th12]: LLVM Project, [Clang Attribute Reference](https://clang.llvm.org/docs/AttributeReference.html#target-clones).

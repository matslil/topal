# Accelerator and DSP patterns

## HA-01 — Bulk SIMT data-parallel kernel

**Good fit.** Many array elements can execute the same mostly pure computation
with regular indexing and limited cross-element coordination.

**References.** CUDA's programming guide defines SIMT kernels and emphasizes
memory throughput and coalescing.[^ha1] Futhark shows compilation of nested
functional parallelism through bulk operators and flattening.[^ha2]

**Hardware assumptions.** GPU/SIMT processor; vector CPU and some NPUs can use
related lowering.

**Problem.** Scalar host loops fail to expose enough independent work or force
manual thread/block index code into the algorithm.

**Structure.** Apply a pure element/tile function across a static or bounded
index space; derive launch geometry; isolate synchronization and host/device
effects at kernel boundaries.

**Limitations.** Branch divergence, irregular work, tiny inputs, kernel launch,
and transfers can erase gains. Global synchronization is restricted.

**Core-language support required.** Bulk array combinators or parallel index
spaces, purity/independence evidence, shapes and bounds, restricted kernel
effects, device-callable specialization, reductions/scans, and backend launch
and transfer semantics.

## HA-02 — Shared/local-memory tiling

**Good fit.** A kernel reuses a bounded tile of data enough to justify staging
it from global memory into explicitly managed fast memory.

**References.** CUDA recommends tiled shared-memory matrix multiplication to
avoid redundant global transfers.[^ha3] MLIR Linalg exposes tiling and promotion
to fast memory as transformations.[^ha4]

**Hardware assumptions.** GPU shared memory, NPU SRAM, DSP scratchpad, or CPU
cache with known capacity/alignment.

**Problem.** Repeated slow-memory access dominates arithmetic and wastes
bandwidth.

**Structure.** Cooperatively load an aligned tile, synchronize, perform all
reuse locally, write results, and repeat with target-tuned dimensions.

**Limitations.** Tiles consume scarce local memory/registers and reduce
occupancy; bank conflicts, halos, and boundary tiles add complexity.

**Core-language support required.** Multiple memory spaces, static tile shapes,
alignment/layout evidence, cooperative groups and barriers, bounded local
allocation, affine indexing, and parameterized target schedules.

## HA-03 — Coalesced tensor/array layout

**Good fit.** Adjacent lanes consume related elements and physical layout can
be selected to minimize memory transactions or match accelerator operators.

**References.** CUDA identifies coalescing as a central kernel performance
consideration.[^ha1] OpenXLA assigns physical layouts and materializes copies
where layout constraints conflict.[^ha5]

**Hardware assumptions.** GPU memory transactions, vector loads, NPU tensor
formats, or banked DSP memories.

**Problem.** A semantically correct dimension/record order can produce strided,
misaligned, bank-conflicting, or copy-heavy access.

**Structure.** Preserve logical shape separately from dimension order, stride,
packing, alignment, and blocking; propagate compatible layout through the graph
and convert only at real boundaries.

**Limitations.** One consumer's ideal layout may hurt another; physical
transposes cost time and memory; target-specific formats reduce portability.

**Core-language support required.** First-class static shapes and explicit
physical layouts, strides/alignment/packing, layout-polymorphic functions,
validated reinterpretation, layout propagation, and target-specific
specialization without changing semantic values.

## HA-04 — Kernel/operator fusion

**Good fit.** A producer's intermediate is consumed locally by compatible
operations, and avoiding materialization is worth a larger combined kernel.

**References.** OpenXLA calls fusion its most important GPU optimization and
uses it to keep intermediates in registers/shared memory rather than HBM.[^ha5]
Futhark defines fusion rules for bulk operators.[^ha2]

**Hardware assumptions.** Particularly GPU/NPU, but loop fusion also benefits
CPU/DSP cache and allocation.

**Problem.** Separate abstraction stages launch extra kernels and write/read
large intermediate arrays.

**Structure.** Retain the computation graph, prove compatible dependencies and
effects, inline producer expressions into consumers, and allocate no observable
intermediate.

**Limitations.** Fusion can increase registers, code size, compilation time,
recomputation, and reduce parallel occupancy; external effects and multiple
consumers limit legality.

**Core-language support required.** Pure/effect-aware graph IR, bulk operations,
shape and alias evidence, nonmaterialized values, aggressive inlining/fusion,
cost-guided target specialization, and observable-allocation semantics that
permit elimination.

## HA-05 — Asynchronous double-buffered transfer/compute

**Good fit.** Data arrives in regular blocks and DMA or asynchronous copies can
overlap the transfer of block N+1 with computation on block N.

**References.** CUDA documents asynchronous global-to-shared copies that
overlap movement and computation.[^ha6] TI documents ping-pong DMA buffering
for continuous DSP data streams.[^ha7]

**Hardware assumptions.** DMA/copy engine plus at least two buffers, independent
compute, completion events, and coherent ownership transitions.

**Problem.** Synchronous transfer leaves compute idle; one buffer risks overwrite
while still in use.

**Structure.** Alternate buffers through free, filling, ready, computing, and
free states; issue the next transfer early and wait only before first use.

**Limitations.** Doubles buffer memory, startup/drain phases remain exposed,
and transfer variability can still stall. Incorrect fencing causes races.

**Core-language support required.** Typestate buffers, linear CPU/device
ownership transfer, asynchronous effects and completion tokens, DMA layouts,
barriers/fences, static capacity, and pipeline scheduling with cleanup on error.

## HA-06 — Algorithm/schedule separation

**Good fit.** One mathematical pipeline needs different loop, tile, vector,
parallel, and storage choices across CPU, GPU, NPU, and DSP targets.

**References.** Halide separates image-processing algorithms from schedules and
demonstrates target-specific high performance on ARM, x86, and GPU.[^ha8] TVM
extends graph/operator scheduling across deep-learning backends.[^ha9]

**Hardware assumptions.** None semantically; schedule vocabulary and legality
are target-dependent.

**Problem.** Embedding optimization order in algorithm code harms readability,
portability, and reuse; fully automatic search may miss expert knowledge.

**Structure.** Define pure mathematical results independently; attach a checked
schedule describing placement, tiling, fusion, vectorization, memory, and
parallel mapping.

**Limitations.** The schedule can be complex, brittle across shapes/targets, and
expensive to search. Schedule correctness and performance are separate proofs.

**Core-language support required.** Stable semantic IR, named schedule
transformations, legality/dependence checks, static shapes/ranges, target
introspection, schedule parameters/autotuning, and a portable unscheduled
fallback usable by interpreter and compiler.

## HA-07 — Quantized integer inference

**Good fit.** Inference accuracy tolerates a calibrated lower-precision
representation and integer hardware offers better latency, footprint, or power.

**References.** Jacob et al. give an integer-only inference quantization scheme
with explicit scale/zero-point reasoning and report reduced footprint.[^ha10]

**Hardware assumptions.** Integer SIMD/DSP/NPU operations, often int8 multiply
with wider accumulation and defined rounding/saturation.

**Problem.** Floating-point models consume excessive bandwidth, storage, and
energy or cannot run on integer-only accelerators.

**Structure.** Associate tensors with quantization parameters; perform scaled
integer operations with specified accumulator width; requantize explicitly;
calibrate or train for accuracy.

**Limitations.** Loses numerical accuracy, may overflow, and can insert costly
requantization between incompatible scales. Benefits depend on hardware support.

**Core-language support required.** Fixed-width integer layouts, scaled/quantized
types, per-tensor/channel static parameters, widening accumulation, explicit
rounding/saturation, tensor shapes, and backend quantized-operator selection.

## HA-08 — Mixed-precision compute with wider accumulation

**Good fit.** Storage and multiplication tolerate narrow floating point while
critical reductions, updates, or master values require wider precision.

**References.** Micikevicius et al. describe mixed-precision training using
half precision with techniques such as wider accumulation and loss scaling.[^ha11]

**Hardware assumptions.** CPU/GPU/NPU with high-throughput narrow arithmetic
and supported conversion/accumulator paths.

**Problem.** Uniform wide precision wastes compute and memory bandwidth;
uniform narrow precision underflows, overflows, or accumulates excessive error.

**Structure.** Assign precision per value/operation, accumulate sensitive sums
wide, retain master state where necessary, and make scaling/conversion explicit.

**Limitations.** Numerical behavior changes; conversion and scale management
cost code and time; target formats differ; deterministic results may be hard.

**Core-language support required.** Exact approximate-number formats and
rounding, explicit conversions, accumulator result typing, mixed-operation
overloads, range/error evidence, reproducible-reduction choices, and backend
hardware capability matching.

## HA-09 — Sparse compressed execution

**Good fit.** Weights or data contain enough zeros/structure that skipping work
and storing indices beats dense regular execution.

**References.** EIE operates directly on compressed sparse neural networks and
uses weight sharing and zero skipping to reduce memory traffic and work.[^ha12]

**Hardware assumptions.** NPU/GPU/CPU/DSP with sparse operators or efficient
irregular gather/metadata processing; structured sparsity matches hardware best.

**Problem.** Dense execution spends bandwidth and arithmetic on zeros and
models may not fit on-chip memory.

**Structure.** Encode nonzeros plus indices/blocks; propagate sparsity metadata;
dispatch a sparse kernel that skips absent work and preserves tensor semantics.

**Limitations.** Index overhead, load imbalance, irregular access, and low
sparsity can be slower than dense code; pruning changes accuracy; formats vary.

**Core-language support required.** Sparse tensor types/layouts with static
structure where possible, shape invariants, zero semantics, compressed views,
specialized iteration/reduction, and target-driven dense/sparse alternatives.

## HA-10 — Static systolic dataflow mapping

**Good fit.** A regular recurrence such as matrix multiplication or convolution
can stream operands and partial results through a repeated processing-element
array with high reuse.

**References.** Kung presents systolic architecture as a methodology for
mapping high-level computations to regular hardware structures with multiple
computations per memory access.[^ha13]

**Hardware assumptions.** NPU/FPGA/special-purpose array with local links,
pipeline registers, and known dimensions/dataflow.

**Problem.** Central memory bandwidth cannot feed all arithmetic units, and
general dynamic scheduling wastes area/energy.

**Structure.** Spatially map a regular iteration domain; stream values at fixed
rates through processing elements; accumulate locally; schedule fill, steady
state, and drain statically.

**Limitations.** Shape mismatch and irregular control reduce utilization;
padding and pipeline latency cost time; mapping is target-specific.

**Core-language support required.** Affine recurrence/dataflow representation,
static shapes and rates, local channels, pipeline delays, placement/schedule
parameters, fixed numerical semantics, and synthesis/accelerator backend
contracts.

## HA-11 — Synchronous dataflow with static schedule

**Good fit.** DSP/stream actors consume and produce a statically known number
of tokens per firing, enabling compile-time buffer and execution schedules.

**References.** Lee and Messerschmitt show that synchronous dataflow's known
rates admit compile-time schedules without runtime scheduling overhead.[^ha14]
Lustre shows synchronous dataflow compilation to efficient sequential code.[^ha15]

**Hardware assumptions.** DSP/MCU/FPGA or streaming CPU/GPU pipeline; bounded
buffers and clocks/rates must be implementable.

**Problem.** Dynamic dataflow requires runtime readiness checks and can have
unbounded buffers or nondeterministic timing.

**Structure.** Give each actor fixed token rates, solve balance equations,
construct a periodic firing schedule, and allocate finite channel buffers.

**Limitations.** Data-dependent rates need richer models and may lose static
schedulability; mode changes and external timing require explicit protocols.

**Core-language support required.** Typed streams, static production/consumption
rates, synchronous ticks/clocks, delay/state operators, balance/deadlock checks,
bounded buffers, static schedule generation, and deterministic actor semantics.

## HA-12 — Fixed-point saturating DSP arithmetic

**Good fit.** Signal/control code needs a known quantum and range, and overflow
should clamp rather than wrap or allocate/promote.

**References.** Arm CMSIS-DSP documents fixed-point Q formats and saturating
arithmetic used by DSP kernels.[^ha16]

**Hardware assumptions.** DSP/MCU/SIMD CPU with fixed-width multiply-accumulate,
widening, rounding, and saturation instructions; portable software fallback.

**Problem.** Floating point may be unavailable, slower, less analyzable, or use
too much memory; mathematical integers do not model register overflow behavior.

**Structure.** Choose scale and signed width statically, widen intermediates,
round explicitly, and saturate at declared bounds; prove ranges where possible.

**Limitations.** Quantization error and saturation distort results; rescaling
costs instructions; incompatible Q formats and hidden implicit conversion cause
bugs.

**Core-language support required.** Fixed-point semantic types separated from
storage layout, fixed-width representations, explicit rounding/saturating and
widening operations, range/unit analysis, vectorizable arrays, and instruction
selection that preserves exact overflow semantics.

## Sources

[^ha1]: NVIDIA, [CUDA Programming Guide — Writing SIMT Kernels](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/writing-cuda-kernels.html).
[^ha2]: T. Henriksen et al., [“Futhark: Purely Functional GPU-Programming with Nested Parallelism and In-Place Array Updates”](https://futhark-lang.org/publications/pldi17.pdf), PLDI, 2017.
[^ha3]: NVIDIA, [CUDA C++ Best Practices Guide](https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/index.html), “Shared Memory in Matrix Multiplication.”
[^ha4]: LLVM Project, [MLIR Linalg Dialect](https://mlir.llvm.org/docs/Dialects/Linalg/).
[^ha5]: OpenXLA, [XLA:GPU Architecture Overview](https://openxla.org/xla/gpu_architecture).
[^ha6]: NVIDIA, [CUDA Programming Guide — Asynchronous Data Copies](https://docs.nvidia.com/cuda/cuda-programming-guide/03-advanced/advanced-kernel-programming.html#asynchronous-data-copies).
[^ha7]: Texas Instruments, [*Ping-Pong Buffering Using DMA*](https://www.ti.com/lit/an/spra641/spra641.pdf), application report SPRA641.
[^ha8]: J. Ragan-Kelley et al., [“Decoupling Algorithms from Schedules for Easy Optimization of Image Processing Pipelines”](https://people.csail.mit.edu/jrk/halide12/), PLDI, 2012.
[^ha9]: T. Chen et al., [“TVM: An Automated End-to-End Optimizing Compiler for Deep Learning”](https://www.usenix.org/conference/osdi18/presentation/chen), OSDI, 2018.
[^ha10]: B. Jacob et al., [“Quantization and Training of Neural Networks for Efficient Integer-Arithmetic-Only Inference”](https://openaccess.thecvf.com/content_cvpr_2018/html/Jacob_Quantization_and_Training_CVPR_2018_paper.html), CVPR, 2018.
[^ha11]: P. Micikevicius et al., [“Mixed Precision Training”](https://openreview.net/forum?id=r1gs9JgRZ), ICLR, 2018.
[^ha12]: S. Han et al., [“EIE: Efficient Inference Engine on Compressed Deep Neural Network”](https://hanlab.mit.edu/projects/eie), ISCA, 2016.
[^ha13]: H. T. Kung, [“Why Systolic Architectures?”](https://www.eecs.harvard.edu/~htk/publication/1982-kung-why-systolic-architecture.pdf), *Computer*, 1982.
[^ha14]: E. Lee and D. Messerschmitt, [“Synchronous Data Flow”](https://ptolemy.berkeley.edu/publications/papers/87/staticscheduling/), *Proceedings of the IEEE*, 1987.
[^ha15]: N. Halbwachs et al., [“The Synchronous Data Flow Programming Language LUSTRE”](https://doi.org/10.1109/5.97300), *Proceedings of the IEEE*, 1991.
[^ha16]: Arm, [CMSIS-DSP Fixed-Point Functions](https://arm-software.github.io/CMSIS-DSP/main/group__FIXED.html).

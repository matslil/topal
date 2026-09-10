# Memory and CPU data-path patterns

## MM-01 — Uniqueness-guided in-place update

**Good fit.** A functional update consumes the last observable reference to a
large array or record, so physical mutation cannot be observed.

**References.** Futhark uses uniqueness types to support in-place array updates
while retaining a purely functional language.[^mm1]

**Hardware assumptions.** General RAM; particularly important for CPU/GPU
arrays and bandwidth-limited kernels.

**Problem.** Naively copying an immutable aggregate for every update can make a
safe functional formulation asymptotically or practically worse than C/CUDA.

**Structure.** Track aliasing/consumption. When the old value is unique and dead,
lower functional reconstruction to writes into its existing storage; otherwise
retain copy-on-write semantics.

**Limitations.** Aliases, retained snapshots, exceptions, and opaque calls may
defeat uniqueness. Reusing storage can retain excessive capacity or secret data.

**Core-language support required.** Move/consumption semantics or provable
uniqueness, lifetime and alias analysis, immutable source values, explicit
effect boundaries, representation-preserving update, and guarantees that
optimization never changes observations.

## MM-02 — Region or arena allocation

**Good fit.** Many objects share a known lifetime or phase and can be reclaimed
together, such as one request, frame, compiler pass, or batch.

**References.** Tofte and Talpin give a type-and-effect discipline that infers
region allocation and deallocation without garbage collection.[^mm2]

**Hardware assumptions.** Byte-addressed memory; works on hosted CPUs and
bounded embedded memories.

**Problem.** Per-object allocation/free adds metadata, fragmentation, and
unpredictable latency; tracing collection adds pauses.

**Structure.** Allocate objects monotonically inside a region; prove none
outlive the region; release the whole region in constant or size-independent
administrative work.

**Limitations.** A long-lived object can retain a whole region; region inference
is conservative; finalizers and cross-region cycles complicate reclamation.

**Core-language support required.** Region/lifetime identity, escape checking,
ownership of the region, allocation-effect tracking, deterministic bulk
cleanup, and optional source control when inference cannot meet a bound.

## MM-03 — Static preallocation and no hot-path allocation

**Good fit.** Memory must be bounded before deployment or allocator latency and
fragmentation are unacceptable in a real-time/safety-critical hot path.

**References.** The Ravenscar profile restricts implicit heap allocation and
other dynamic facilities for analyzable high-integrity tasking.[^mm3]

**Hardware assumptions.** Especially MCU/DSP/bare metal and hard real-time CPU;
also useful for exchange and packet-processing loops.

**Problem.** Dynamic allocation introduces failure modes, synchronization,
fragmentation, cache churn, and latency tails.

**Structure.** Determine maximum capacities statically, allocate buffers and
task state during construction, and reuse bounded slots during steady state.

**Limitations.** Reserves worst-case memory, rejects unbounded workloads, and
needs explicit overload policy. “No heap” does not itself bound stack use.

**Core-language support required.** Static sizes/capacities, fixed-layout
storage, a transitive no-allocation effect/guarantee, bounded recursion and
stack analysis, controlled initialization phases, and typed capacity failure.

## MM-04 — Zero-copy owned region with validated views

**Good fit.** Parsers, protocol stacks, storage, and DMA pipelines need several
typed interpretations of one buffer without copying payload bytes.

**References.** Rust documents safe disjoint slicing through borrow splitting.[^mm4]
The Linux DMA API documents ownership transitions between CPU and device.[^mm5]

**Hardware assumptions.** Shared or mapped memory; DMA adds cache-coherency,
alignment, pinning, and addressability constraints.

**Problem.** Repeated decode/copy stages consume bandwidth and add latency;
unbounded pointer views create use-after-free and aliasing faults.

**Structure.** One region owns bytes. Bounded spans borrow subranges; validation
adds typed view evidence. Ownership explicitly transfers between pipeline
stages or CPU/device domains.

**Limitations.** Holding a tiny view can retain a large buffer; device and CPU
coherence may require synchronization; scatter/gather limits and alignment can
force copies.

**Core-language support required.** Owned buffers, bounded non-owning views,
lifetime and disjointness proofs, explicit layouts, zero-copy decoding,
scatter/gather values, and typed CPU/device ownership and synchronization.

## MM-05 — Structure-of-arrays / AoSoA layout

**Good fit.** A loop processes the same fields across many records and SIMD or
cache bandwidth matters.

**References.** Intel recommends AoS-to-SoA/AoSoA transformations to avoid
gather/scatter and improve vectorization when access patterns warrant it.[^mm6]

**Hardware assumptions.** SIMD/vector CPU, GPU coalescing, or vector DSP; tile
width often depends on the target.

**Problem.** Array-of-structures layout fetches unused fields and turns unit-
stride vector loads into gathers/scatters.

**Structure.** Store hot fields in independent contiguous arrays, or small
vector-width field blocks inside an AoSoA, while preserving a semantic record
view.

**Limitations.** SoA worsens per-record locality, increases bookkeeping, and can
make boundary interoperability costly. One layout is not optimal for all uses.

**Core-language support required.** Separation of semantic and physical type,
field/array layout controls, static vector/tile sizes, safe views over alternate
layouts, alignment/stride evidence, and target-specific specialization.

## MM-06 — Cache-aware loop tiling

**Good fit.** Dense loops repeatedly reuse subsets of arrays or tensors that can
fit in a cache or scratchpad.

**References.** Intel describes blocking transformations for loops whose
working set otherwise exceeds cache.[^mm7] MLIR Linalg treats tiling and
promotion to fast memory as first-class transformations.[^mm8]

**Hardware assumptions.** CPU caches, GPU shared memory, NPU SRAM, or DSP
scratchpad with finite capacity.

**Problem.** A correct loop nest may reload the same data from a slow memory
level and become bandwidth-bound.

**Structure.** Partition iteration domains into tiles, execute all profitable
work on a tile while resident in faster memory, and handle boundary tiles
without violating dependencies.

**Limitations.** Optimal sizes are target- and shape-dependent; extra boundary
control can hurt small inputs; irregular access and aliasing obstruct reuse.

**Core-language support required.** Affine/bounded iteration or analyzable bulk
operators, explicit shapes/strides, no-alias and dependence evidence, local
memory spaces, parameterized schedules, and safe residual/boundary semantics.

## Sources

[^mm1]: T. Henriksen et al., [“Futhark: Purely Functional GPU-Programming with Nested Parallelism and In-Place Array Updates”](https://futhark-lang.org/publications/pldi17.pdf), PLDI, 2017.
[^mm2]: M. Tofte and J.-P. Talpin, [“Region-Based Memory Management”](https://doi.org/10.1006/inco.1996.2613), *Information and Computation*, 1997.
[^mm3]: A. Burns, B. Dobbing, and T. Vardanega, [*Guide for the Use of the Ada Ravenscar Profile in High Integrity Systems*](https://www.open-std.org/jtc1/sc22/wg9/n435.pdf), 2003.
[^mm4]: Rust Project, [Splitting Borrows](https://doc.rust-lang.org/nomicon/borrow-splitting.html).
[^mm5]: Linux kernel, [Dynamic DMA mapping Guide](https://docs.kernel.org/core-api/dma-api-howto.html).
[^mm6]: Intel, [Memory Layout Transformations](https://www.intel.com/content/www/us/en/developer/articles/technical/memory-layout-transformations.html).
[^mm7]: Intel, [Loop Optimizations Where Blocks Are Required](https://www.intel.com/content/www/us/en/developer/articles/technical/loop-optimizations-where-blocks-are-required.html).
[^mm8]: LLVM Project, [MLIR Linalg Dialect](https://mlir.llvm.org/docs/Dialects/Linalg/).

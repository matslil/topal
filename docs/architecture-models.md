# Architecture and platform models

Topal compilers select implementations using a named, versioned description of
the target on which generated code will run. The description covers processors,
accelerators, memories, communication mechanisms, interconnects, and their
composition on a board or platform. It supplies hardware legality facts and
conditional cost information without changing the semantic value computed by a
Topal program.

Architecture descriptions are written in Topal. They use ordinary Topal
composition, units, products, variants, constraints, and functions rather than
a separate configuration language. The compiler loads them through a privileged
architecture-provider boundary; application source does not import the selected
model as a value and cannot branch on compilation decisions. The exact schema,
validation rules, canonical identity, and initial profiles are specified in the
next architecture-model increment.

“Varying architectures” means that different model packages can describe x86,
Arm, RISC-V, GPUs, NPUs, DSPs, and combinations of them. It is not a Topal
language variant and does not give those families one artificial common ISA.

## Purposes and boundaries

An architecture model supports four related decisions:

1. reject a lowering which the selected target cannot execute correctly;
2. compare already legal implementation plans using explicit target costs;
3. produce exact target, model, and assumption identities in artifacts and
   implementation evidence; and
4. explain which missing program or architecture facts prevented a decision.

It does not define Topal semantics, grant access to a device or address, replace
an ABI description, or prove that an empirical estimate is universally true.
Language legality, effects, failures, ownership, information flow, and resource
guarantees are checked before profitability. A provider can establish a target
fact only through the checked evidence boundary described in
[contracts and evidence](contracts-and-evidence.md).

The model is descriptive rather than imperative. It describes that a DMA engine
can perform a transfer under stated conditions and what that transfer costs; it
does not execute DMA, inspect the compilation host, or call a vendor tool. Such
observations enter through separately qualified providers and calibration
records.

## Layered identity

A complete compilation target is a composition of independently identified
layers. Keeping the layers separate avoids treating a vendor, ISA, processor,
operating system, and board as interchangeable names.

### Instruction-set and execution architecture

An execution architecture describes instruction semantics and architectural
state:

- architecture family, revision, profiles, and extension dependency rules;
- scalar, vector, predicate, matrix, accumulator, control, and system state;
- supported operations, types, encodings, addressing modes, and exceptions;
- endian and address-size choices;
- architectural memory accesses, atomics, ordering scopes, and barriers;
- privilege levels, traps, interrupts, and architected discovery mechanisms;
  and
- compatibility and feature implication rules.

An ISA fact answers whether an operation is available and what it means. It is
not a latency claim. Intel and AMD products can implement the same x86-64 ISA
baseline with different microarchitectural costs; Arm and RISC-V profiles can
likewise have many implementations.

### ABI and platform

A platform layer selects the data layout, calling and syscall conventions,
object format, relocation model, executable entry, loader policy, and available
typed platform boundaries. ABI and platform identity remain explicit even when
two platforms use the same ISA and processor.

Architecture descriptions may refer to a separately qualified ABI model but do
not silently derive one from an ISA name. Foreign adapters, persistent layouts,
and Topal's private representation remain independent decisions.

### Microarchitecture and compute elements

A compute-element model describes one implementation of an execution
architecture. A platform can contain heterogeneous instances. CPU cores, GPU
compute units, NPU tiles, DSP lanes, management processors, and fixed-function
engines are all compute elements rather than exceptional top-level cases.

Relevant facts include:

- supported architecture features and any per-element differences;
- issue, dispatch, retirement, and pipeline resources;
- instruction latency, reciprocal throughput, micro-operation decomposition,
  resource occupancy, hazards, and permitted parallel issue;
- register files, banks, classes, aliases, ports, and spill paths;
- branch prediction and indirect-branch resources where publicly modelable;
- scalar, SIMD, scalable-vector, SIMT, tensor, matrix, saturation, and fused
  operations;
- local memory, scratchpad, queues, and per-dispatch resource limits;
- hardware thread, core, cluster, socket, warp, wave, and tile topology;
- frequency, power, thermal, and operating-mode conditions; and
- launch, synchronization, preemption, and completion mechanisms.

Instruction storage cost means encoded code bytes and required literal/table
bytes. Dynamic stack, spill, register, scratchpad, and temporary-buffer usage
are separate resource dimensions. “Instruction cost” is never one universal
number: latency, throughput, code size, energy, and occupied resources can favor
different sequences.

### Memory and address spaces

A memory element identifies an addressable or transferable storage domain and
records:

- medium and technology class without assuming that the class alone determines
  performance;
- capacity, address range, addressing granularity, and supported access sizes;
- alignment, boundary, burst, row, bank, rank, channel, and interleave rules;
- read, write, atomic, execute, persistence, volatility, and privilege
  properties;
- static and dynamic latency and bandwidth conditions;
- ports, outstanding-request limits, queues, and arbitration;
- error detection/correction and explicit failure behavior;
- ownership, visibility, and coherence domains; and
- accessibility from each compute or transfer element.

Virtual address translation is modeled separately from backing memory. Page
sizes, page-walk levels, translation caches, address-space identifiers, IOMMU
translation, pinning, and mapping costs can therefore be composed with memory
access rather than hidden in a nominal DRAM latency.

Heterogeneous memories retain distinct identities. CPU DRAM, high-bandwidth
device memory, nonvolatile memory, on-chip SRAM, DSP scratchpad, register files,
and device-local constant memory need not share addressability, coherence,
failure, or allocation rules.

### Cache and coherence hierarchy

A cache model records what it can cache—addresses, instructions, data,
translations, descriptors, or another named object—and describes:

- capacity, line or sector size, set count, associativity, indexing and known
  partitioning;
- private/shared scope and topology attachment;
- write, allocation, inclusion, exclusivity, and replacement properties;
- hit paths, miss paths, fill/writeback traffic, and outstanding misses;
- prefetch mechanisms and their stated applicability;
- coherence participation, visibility domain, ownership states, and snoop or
  directory paths; and
- maintenance, invalidation, flush, and synchronization operations.

Cache utilization is not stored as a property of the cache. It is an analysis
result derived from program access regions and order, physical layout, mapping,
working-set size, concurrent users, and cache organization. Unknown replacement
or indexing details remain unknown. False sharing is derived when independently
accessed or written locations occupy one coherence unit; the cache-line and
coherence topology come from the model while access ownership comes from the
program plan.

Hardware cache coherence is described by its guaranteed scope and observable
ordering consequences, not by assuming a named protocol behaves identically on
every product. Noncoherent participants require explicit ownership transfer and
maintenance operations.

### Interconnect and bus topology

Compute, memory, cache, DMA, channel, and device instances are nodes in a
directed connectivity graph. Links and routing choices record:

- supported initiators, targets, address or message classes, and routes;
- transfer granularity, width, burst, packet, and alignment rules;
- one-way latency, bandwidth, duplex behavior, and protocol overhead;
- queue capacity, credits, outstanding transactions, arbitration, and priority;
- ordering, atomicity, coherency, broadcast, and failure properties;
- clock or power-domain crossings; and
- resources shared with other routes.

A path cost composes link and endpoint costs while retaining shared resource
identities. Competing transfers using the same channel, controller, switch,
cache-fill path, or memory bank can therefore expose a bottleneck. Bandwidths
are not simply added when traffic shares a narrower parent link, and latency is
not treated as throughput.

### DMA and hardware communication channels

A DMA or copy engine records its reachable source and destination spaces,
address width, descriptor format, alignment, granularity, maximum transfer,
scatter/gather limits, queue depth, setup cost, concurrency, completion and
failure mechanisms, and relationship to caches and IOMMUs. It also states
whether CPU/device ownership transfer, flush, invalidate, barrier, or pinning is
required before and after the operation.

Hardware communication channels include FIFOs, mailboxes, doorbells, stream
ports, interprocessor interrupts, shared queues, and on-chip networks. Their
models distinguish payload capacity, backpressure, ordering, atomicity,
visibility, notification, sender/receiver topology, and completion semantics.
A low modeled transfer cost cannot substitute for missing protocol or ownership
proof.

### Board and deployment composition

A board model instantiates compute, memory, cache, interconnect, transfer,
clock, power, interrupt, and external-interface components and connects them by
identity. It can describe external caches, memory controllers, NUMA nodes,
PCIe-like fabrics, device buses, memory-mapped devices, and board-specific
bandwidth or capacity limits without modifying a reusable processor definition.

Overlays may add populated capacities, clock modes, qualified measurements, or
errata constraints. They cannot contradict a base model silently. Every
override names the replaced fact, its authority, applicability, and resulting
model identity.

This extensibility prepares later systems work involving MMIO, privileged
execution, interrupts, devices, and boot platforms. The architecture model by
itself does not claim that the current Topal language or compiler can build an
operating-system kernel.

## Access barriers and ordering

Barrier records distinguish at least:

- compiler ordering barriers;
- memory fences by read/write class and ordering strength;
- acquire, release, and read-modify-write operations;
- execution or instruction-synchronization barriers;
- cache and translation maintenance completion;
- device or DMA ownership and visibility barriers; and
- synchronization scope, such as thread, core, cluster, device, system, or a
  named topology domain.

The model states architectural guarantees, required preconditions, affected
domains, and cost conditions. It does not expose barrier selection in portable
Topal source. A compiler-synthesized mechanism may use a barrier only after the
memory and concurrency specifications establish that the complete mechanism is
semantically valid.

## Conditional multidimensional costs

Cost information is a set of conditional facts, not a scalar attached to a
component. A cost query identifies:

- operation or transfer identity;
- operand/result types and sizes;
- access size, alignment, stride, layout, and address spaces;
- dependency shape and available parallelism;
- locality or cache-state assumption;
- concurrency, contention, occupancy, and queue assumptions;
- operating mode, frequency, power, and temperature conditions;
- input shape or value-range conditions where relevant; and
- the model and calibration identities supplying the answer.

The result keeps independently comparable dimensions such as:

- latency and initiation interval;
- sustainable and peak throughput or bandwidth;
- encoded code and static-data size;
- register, stack, local, temporary, peak-live, and retained memory;
- transfer count and bytes by route;
- occupied ports, banks, queues, engines, and compute resources;
- launch, setup, synchronization, and cleanup work;
- energy or power when qualified evidence exists; and
- predictability information needed for worst-case or tail analysis.

Quantities use exact Topal units. A fact may be exact, a lower/upper interval,
a piecewise function, or an estimate with an error/confidence description.
Unknown is distinct from zero, unbounded, and unsupported. A hard capacity,
legality, deadline, or safety condition is never converted into a favorable
weighted estimate.

Latency and throughput compose differently. Parallel resources can overlap;
dependent operations cannot. Memory traffic competes on shared paths. Cache
and prediction effects depend on workload state. The model therefore exposes
resource and topology identities so the implementation-plan analysis can
compose costs rather than blindly summing nominal instruction times.

## Provenance, calibration, and confidence

Every fact records its source class and applicability:

- architecture or vendor specification;
- qualified board/platform description;
- toolchain/backend model;
- reproducible static derivation;
- controlled measurement or benchmark calibration;
- administrator-supplied deployment constraint; or
- explicit conservative assumption.

Calibration is an immutable overlay tied to the exact hardware, firmware,
operating mode, tool, method, workload shape, and date. Measurements do not
rewrite the reusable architectural definition. Conflicting sources remain
visible, and qualification policy determines which may establish verified,
trusted-unverified, or externally-assumed implementation evidence.

An estimate's confidence concerns prediction quality, not semantic truth. Low
confidence can make two alternatives incomparable or select a conservative
fallback; it cannot make an unsupported instruction legal.

## Generic, specific, and cross targets

Target selection distinguishes the compilation host from the execution target.
This section defines the target-selection design once multiple architecture
profiles are qualified. The current bootstrap compiler remains restricted to
the qualified Linux x86-64 baseline described in
[the build system](build-system.md) and rejects other targets until their
profiles, lowering, and validation exist.

The normal implicit choice is the generic baseline model for the compilation
host's architecture family and platform. Detecting that the compiler runs on an
x86-64 or AArch64 host selects the corresponding generic baseline; it does not
silently enable every feature of that particular processor. The artifact
records the resolved target identity, so caches and reproducible builds do not
treat outputs from different host families as equal.

An explicit specific target selects a named architecture, processor or
microarchitecture, feature set, platform, and optionally board. It may use
special instructions only within that declared compatibility envelope. An
explicit native-tuning mode may ask a qualified provider to detect more of the
current host, but that behavior is never implied by the generic default.

Cross-compilation explicitly selects a foreign target model and does not query
the compilation host for target performance or features. Missing platform,
board, calibration, or workload facts remain unknown and can cause conservative
fallbacks or diagnostics.

Runtime multiversioning is a separate plan: the artifact contains a baseline
implementation, one or more compatible specialized implementations, and a
qualified detection/dispatch mechanism. Each version has the same Topal
semantics. Variant count, code size, startup, detection reliability, and
dispatch overhead are costed explicitly; a dynamic loader or foreign runtime is
never acquired implicitly.

## Topal authoring and bootstrap boundary

Architecture packages are Topal text and use a versioned standard schema. They
are declarative, total, deterministic, and authority-free. Validation evaluates
only the restricted static construction needed to build the model. It performs
no device access, host discovery, file search, network request, arbitrary
effect, or target code execution.

This avoids a bootstrap cycle: the model loader validates the schema-level
Topal description using target-independent shared syntax and semantics before
selecting a backend. It does not compile and run the architecture package using
the architecture that package is defining.

Canonical model identity includes schema and language revisions, canonical
package contents, imports, overlays, provenance records, and all facts that can
affect legality or cost. Cosmetic source changes may have a separate source
identity, but two different effective facts cannot share one model identity.

The compiler and diagnostic tools can project selected facts and decisions for
people to inspect. That projection is read-only and cannot be edited into proof
or plan input.

## Optimization decisions supported

The model is designed to inform the research catalog without assigning policy
weights. Examples include:

| Decision | Principal architecture information |
| --- | --- |
| instruction selection and combining | ISA semantics, feature envelope, encoding size, latency, throughput, resources |
| scheduling and register pressure | dependencies, pipelines, ports, hazards, register files, spill paths |
| SIMD/SIMT/tensor lowering | legal shapes, lanes, masks, gathers, reductions, divergence, matrix units |
| unrolling and multiversioning | code-size/cache limits, registers, feature distribution, dispatch cost |
| layout, tiling, fusion, and buffer reuse | shapes, cache/scratch capacity, banks, transactions, occupancy, conversion paths |
| memory placement and prefetch | memory hierarchy, addressability, translation, access patterns, queues, bandwidth |
| DMA and asynchronous staging | reachable spaces, routes, setup, overlap, ownership, coherence, completion |
| accelerator mapping and library selection | compute topology, launch geometry, supported operators, ABI, transfers, workspace |
| communication and concurrency mechanisms | channel capacity, ordering, atomics, barriers, coherence and topology scopes |
| code and data layout | encoding sizes, branch ranges, instruction caches, pages and translation |

Program facts remain equally necessary. A cache model alone cannot predict
reuse; a SIMD unit alone cannot prove independent iterations; and a DMA engine
alone cannot prove that ownership transfer preserves program behavior.

## Compatibility and qualification

Model imports and overlays use exact revisions. A consumer rejects unknown
required schema elements, contradictory feature sets, impossible topology,
dimensionally invalid quantities, dangling component identities, unsupported
cost functions, and facts whose authority is insufficient for their use.

Qualification distinguishes at least:

- schema-valid: internally well-formed and dimensionally consistent;
- implementation-qualified: usable by a named compiler/backend version for
  legality and code generation;
- cost-qualified: tested against stated static or measured tolerances for named
  operations and workloads; and
- platform-qualified: demonstrated to produce and run conforming artifacts on
  the named platform or board.

A generic model may be implementation-qualified while intentionally providing
only conservative costs. A calibration overlay may improve estimates without
changing the set of legal instructions. No model is assumed complete merely
because it is accepted: coverage and unknown fields are reported explicitly.

## Diagnostics and missing information

When missing information changes an optimization decision, a diagnostic
projection identifies:

- the considered transformation and semantic location;
- the missing program, target, board, calibration, or workload fact;
- the alternatives which remained legal;
- the conservative choice made, or the hard requirement which could not be
  proved;
- the model and policy identities used; and
- how an authorized source or model provider could supply the fact.

Diagnostics do not ask application code to assert hardware truth or proof
status. A programmer may add semantic information such as a bound, layout,
effect, or guarantee; a model maintainer or qualification provider supplies
physical facts.

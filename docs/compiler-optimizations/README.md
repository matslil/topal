# Compiler optimization research library

This directory is a non-normative research catalog. It records optimization
ideas found in production compilers and compiler research so later Topal design
can make explicit, source-backed choices. An entry does not mean that Topal
implements, promises, or has approved the optimization. Authoritative Topal
semantics remain in the parent `docs/` documents.

The survey is representative rather than exhaustive. It covers conventional
ahead-of-time compilers (LLVM/Clang and GCC), managed and just-in-time systems
(HotSpot), a compiler with production profile feedback (Go), and specialized or
research systems including Polly, MLIR, Halide, TVM, XLA, Souper, STOKE,
equality saturation, MLGO, and BOLT. Sources were checked on 2026-09-29.
Primary papers, project documentation, and implementation documentation are
preferred over secondary descriptions.

## Systems surveyed

| System | Kind represented here | Representative findings |
| --- | --- | --- |
| LLVM/Clang | widely deployed optimizing compiler infrastructure | SSA simplification, loop/vector passes, interprocedural optimization, target code generation, PGO, and function multiversioning |
| GCC | widely deployed ahead-of-time compiler | optimization-level policy, IPA/LTO, vectorization, machine scheduling, layout, PGO, and multiversioning |
| HotSpot | widely deployed managed/JIT compiler | escape analysis, scalar replacement, tiering, profiles, and speculation |
| Go compiler | widely deployed ahead-of-time compiler | profile-guided inlining and guarded devirtualization |
| Cranelift | production low-latency code generator | bounded acyclic e-graph optimization and explicit compilation-time/code-quality trade-offs |
| Polly and MLIR | production/research compiler infrastructure | polyhedral scheduling, structured tiling/fusion, bufferization, memory spaces, vector and GPU lowering |
| XLA | production specialized compiler | graph fusion, layout propagation, memory planning, library selection, and accelerator code generation |
| Halide and TVM | research-origin specialized compilers in practical use | algorithm/schedule separation, target schedules, empirical search, and latency hiding |
| Souper and STOKE | research superoptimizers | solver-backed IR synthesis and stochastic machine-code search |
| `egg` | research optimization infrastructure | equality saturation with analysis and cost-based extraction |
| MLGO | research integrated into LLVM infrastructure | learned inlining and register-allocation policies with versioned corpora/models |
| BOLT | production-oriented post-link optimizer | profile-guided final-binary block and function layout |

## Catalog

| Family | Entries | Main decision scope |
| --- | --- | --- |
| [Scalar and control flow](scalar-and-control-flow.md) | `CO-SC-001`–`CO-SC-006` | expressions, SSA values, stores, and branches |
| [Loops and memory](loops-and-memory.md) | `CO-LM-001`–`CO-LM-008` | repetition, locality, vector execution, and storage |
| [Interprocedural and feedback](interprocedural-and-feedback.md) | `CO-IF-001`–`CO-IF-006` | calls, objects, whole programs, and observed behavior |
| [Target and heterogeneous lowering](target-and-heterogeneous.md) | `CO-TH-001`–`CO-TH-008` | instruction sets, pipelines, memory spaces, and accelerators |
| [Search, learning, and post-link](search-learning-and-post-link.md) | `CO-SP-001`–`CO-SP-006` | large choice spaces and final binary layout |

The related [design-pattern research library](../design-patterns/README.md)
starts from application structures and asks what language support they need.
This library starts from compiler transformations and asks what problem,
evidence, target facts, and trade-offs each transformation has. Links between
them identify overlap without making either catalog normative.

## Entry schema

Every entry has a permanent ID and the following fields:

| Field | Required content |
| --- | --- |
| **Found in** | Primary source and whether the example is production, specialized, or research work. |
| **Problem** | The performance or resource problem being addressed. |
| **Approach** | The transformation or decision, including its compiler phase. |
| **Assumptions and evidence** | Semantic legality conditions, program facts, target facts, and profile/model inputs. |
| **Limitations and interactions** | Cases where the idea loses, cannot apply, increases risk, or changes opportunities for another pass. |
| **Characteristic weighting** | How execution time, memory traffic, peak memory, code size, compilation cost, startup, energy, or predictability compete. |
| **Topal relevance** | Likely owner and evidence boundary; this is research guidance, not an implementation decision. |

The `CO` prefix means compiler optimization. The family component is `SC`
(scalar/control flow), `LM` (loops/memory), `IF` (interprocedural/feedback),
`TH` (target/heterogeneous), or `SP` (search/post-link). IDs are never
renumbered; a removed entry keeps a tombstone pointing to its replacement.

## Reading cost claims

Legality and profitability are separate. A transformation may be proven to
preserve Topal observations and still be a poor choice on a particular target
or workload. Conversely, a favorable hardware cost estimate never authorizes a
semantic rewrite that lacks the required effect, alias, range, ownership,
numerical, or concurrency evidence.

Costs are conditional on a model identity and its assumptions. Static
instruction counts, modeled cycles, profile counts, benchmark measurements,
and asymptotic resource guarantees are not interchangeable. Missing facts must
remain unknown rather than be assigned a favorable zero cost. Later policy can
turn proven legal alternatives into hard constraints and ordered preferences;
this catalog does not assign universal weights.

## Survey boundaries

The catalog includes optimizations that can inform a Topal-owned semantic or
implementation-plan pass, LLVM IR construction, LLVM's standard pipelines,
target code generation, or a later post-link stage. It excludes:

- transformations whose only claimed benefit is for a language semantic model
  incompatible with Topal;
- hardware tuning claims without a reproducible source or stated target;
- correctness techniques except where they establish an optimization's
  legality; and
- a complete list of every pass flag exposed by a surveyed compiler.

The catalog should grow by adding evidence-backed entries, not by treating a
compiler's option list as a design checklist.

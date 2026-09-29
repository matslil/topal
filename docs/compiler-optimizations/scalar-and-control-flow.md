# Scalar and control-flow optimizations

## CO-SC-001 — Sparse conditional constant propagation

**Found in.** LLVM documents sparse conditional constant propagation (`sccp`)
and interprocedural SCCP among its production transformation passes.[^sc1]

**Problem.** Values and branches that are constant in the reachable program can
leave unnecessary computation and unreachable control flow.

**Approach.** Propagate constants while simultaneously discovering executable
control-flow edges. Replace proven values, simplify branches, and let dead-code
and CFG cleanup remove the newly unreachable remainder.

**Assumptions and evidence.** Operations must be safe to evaluate or reason
about statically, and the lattice must conservatively represent unknown,
constant, and conflicting values. Effects, failures, poison-like IR states,
volatile access, and externally changeable data prevent folding.

**Limitations and interactions.** The analysis is conservative across opaque
calls and dynamic dispatch. Inlining and specialization expose constants;
canonicalization and dead-code elimination consume its results. Repeating it
can improve code but increases compilation time.

**Characteristic weighting.** Usually favors execution time and code size at
modest analysis cost. Duplicating a function for interprocedural constants
trades code size and compilation time for stronger propagation.

**Topal relevance.** Topal can propagate exact static values and evidence in a
semantic plan; LLVM should receive explicit constants and may perform ordinary
SSA SCCP after semantic lowering.

## CO-SC-002 — Algebraic canonicalization and instruction combining

**Found in.** LLVM's production `instcombine` pass combines instructions into
canonical forms, while `reassociate` rearranges associative expressions to
expose constant propagation and redundancy.[^sc1]

**Problem.** Equivalent expressions written or produced in different shapes
hide common subexpressions, constants, and target idioms from later passes.

**Approach.** Apply local identities, fold constants, reassociate permitted
operators, and choose a small set of canonical expression forms.

**Assumptions and evidence.** Every rewrite needs the exact arithmetic and
failure semantics of the operator. Reassociation is invalid when overflow,
rounding, NaNs, exceptions, ordering, or user-visible evaluation is changed
unless corresponding evidence permits it.

**Limitations and interactions.** Greedy rewrites can cycle without a canonical
order and can hide a target-specific pattern. Canonicalization is primarily an
enabler; its isolated benefit may be zero. Exact arbitrary-precision arithmetic
has different profitable identities from fixed-width machine arithmetic.

**Characteristic weighting.** Prefers simpler IR and downstream opportunity,
then instruction count. It should cap rewrite work and avoid increasing code or
live ranges unless a later benefit is supported.

**Topal relevance.** Topal must guard law-based rewrites with numeric, effect,
and evaluation-order evidence. Target-neutral canonical IR can then allow LLVM
to apply its own machine-semantic combines.

## CO-SC-003 — Global value numbering and partial redundancy elimination

**Found in.** LLVM documents global value numbering (`gvn`) as eliminating fully
and partially redundant instructions and redundant loads.[^sc1] GCC enables
global common-subexpression and tree partial-redundancy passes in its optimized
production pipelines.[^sc2]

**Problem.** A value may be recomputed on multiple paths even though an equal
available value already exists, or an expression may be redundant on only some
incoming paths.

**Approach.** Assign equivalence identities to expressions and memory values.
Reuse a dominating result, or place a computation so it becomes available on
all relevant paths and remove later repetitions.

**Assumptions and evidence.** Equality requires compatible types and exact
operator semantics. Load elimination additionally needs alias, memory-version,
volatile, atomic, and effect information proving no intervening change.

**Limitations and interactions.** PRE can introduce computation on a path that
did not previously execute it, increasing work, register pressure, or failure
exposure unless speculation is safe. Alias uncertainty sharply limits memory
GVN.

**Characteristic weighting.** Trades fewer dynamic computations and loads
against longer live ranges, inserted computations, compile-time analysis, and
possible code growth. Edge frequencies can distinguish hot savings from cold
extra work.

**Topal relevance.** Immutable values make value equivalence promising, but
effects and explicit failures constrain speculation. LLVM should receive
accurate alias and memory-effect facts rather than Topal duplicating low-level
memory GVN.

## CO-SC-004 — Dead code, dead store, and unused allocation elimination

**Found in.** LLVM provides ordinary and aggressive dead-code elimination and
dead-store elimination; its pass documentation describes aggressive DCE as
assuming values dead until proven live.[^sc1] HotSpot uses escape analysis and
scalar replacement to eliminate allocations and associated locks.[^sc3]

**Problem.** Lowering, specialization, or earlier transformations can leave
computations, stores, allocations, and synchronization whose results cannot be
observed.

**Approach.** Trace observable roots backward, remove unused pure operations,
remove overwritten or unobservable stores, and erase allocations whose object
identity and effects do not escape.

**Assumptions and evidence.** The compiler needs precise effects, liveness,
aliasing, object identity, cleanup, failure, volatile/MMIO, atomic, and debug
observation rules. Allocation removal requires allocation itself not to be an
observable success, failure, or resource event.

**Limitations and interactions.** Debug builds may retain locations or objects.
Opaque calls, finalization, weak references, address observation, and resource
accounting can make apparently unused work observable. Removing work can alter
timing but not a defined semantic trace.

**Characteristic weighting.** Usually improves time, code size, memory traffic,
and peak memory together. More aggressive analysis costs compilation time and
may reduce debuggability; retaining debug-only shadows can add storage back.

**Topal relevance.** Topal's effect and resource model must decide which work is
observable. Topal-owned planning can erase semantic allocations only with
proof; LLVM can remove lower-level dead SSA operations and stores.

## CO-SC-005 — Control-flow simplification, threading, and if-conversion

**Found in.** LLVM documents CFG simplification and branch-threading passes;
GCC's optimized pipelines include jump threading, switch conversion, block
reordering, and tail merging.[^sc1][^sc2]

**Problem.** Redundant branches, empty blocks, repeated tests, or poorly shaped
control flow add branch overhead and obscure straight-line optimization.

**Approach.** Fold known conditions, merge compatible blocks, bypass blocks
whose outcome is implied, convert suitable branches to selects or predication,
and lower dense decisions to tables when profitable.

**Assumptions and evidence.** Merging or predicating operations must preserve
evaluation, failure, effect, and memory order. Profitability needs branch
probability, misprediction cost, predication support, table density, and code
layout information.

**Limitations and interactions.** Predication executes work from paths that
were previously skipped; jump tables consume data and may harm security or
position independence. Threading can duplicate blocks and inflate code. A
simpler CFG can help vectorization but reduce source-debug fidelity.

**Characteristic weighting.** Balances branch latency and predictability
against extra executed work, code/data size, instruction-cache pressure, and
side-channel policy. Hot biased branches and short balanced branches favor
different choices.

**Topal relevance.** Topal should retain once-only evaluation and explicit
failure/effect boundaries. LLVM can choose ordinary CFG forms when the emitted
IR makes those constraints visible.

## CO-SC-006 — Strength reduction and induction simplification

**Found in.** LLVM documents induction-variable canonicalization and loop
strength reduction, including replacing repeated indexed address calculations
with recurrences.[^sc1]

**Problem.** A loop may repeatedly execute an expensive operation even though
successive values have a cheaper recurrence, or may use a control form that is
difficult to analyze.

**Approach.** Canonicalize induction variables and replace repeated multiply,
power, or address expressions with increments or target-cheaper equivalents.

**Assumptions and evidence.** The recurrence must have identical overflow,
rounding, range, and termination behavior. Profitability depends on instruction
latencies, addressing modes, register availability, and downstream vector form.

**Limitations and interactions.** A recurrence lengthens dependencies and live
ranges, which can be slower on wide out-of-order or vector machines. Fixed-width
overflow identities do not automatically apply to Topal's exact `Int`.

**Characteristic weighting.** Trades operation latency for dependency depth,
register pressure, code complexity, and sometimes numerical stability. The
target scheduler and vectorizer may prefer recomputation over a serial chain.

**Topal relevance.** Topal can expose exact ranges and loop structure, but LLVM
should normally choose machine-level induction and addressing forms from target
costs.

## Sources

[^sc1]: LLVM Project, [LLVM's Analysis and Transform Passes](https://llvm.org/docs/Passes.html).
[^sc2]: GNU Project, [Optimize Options](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html).
[^sc3]: Oracle, [Java HotSpot Virtual Machine Performance Enhancements](https://docs.oracle.com/en/java/javase/17/vm/java-hotspot-virtual-machine-performance-enhancements.html).

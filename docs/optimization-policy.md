# Compiler optimization policy

Topal optimization chooses among implementations which already have the same
required language meaning. It never repairs an invalid program, weakens a
contract, assumes an effect away, or turns favorable cost evidence into proof
of target legality. Mandatory semantic lowering therefore runs at every
optimization level; an optional optimization may only choose a different
conforming implementation plan.

The policy is deliberately separate from the
[architecture model](architecture-models.md). A model describes facts about a
target and their provenance. The policy describes how the compiler uses those
facts, program facts, and user goals to select a plan. Keeping the two separate
allows the same model to support different deployment priorities without
editing hardware truth.

## Decision order

For each optimization decision, the compiler follows this order:

1. Construct candidate plans whose semantic preconditions are proved.
2. Reject candidates which need unavailable instructions, ABI behavior,
   address spaces, ordering, alignment, capacity, or other target facilities.
3. Reject candidates which violate a user hard limit, a language resource
   guarantee, a real-time condition, a security constraint, or an applicable
   board constraint.
4. Compare the remaining plans using applicable cost evidence.
5. Select a deterministic result using the active ordered goals and the final
   stable tie-break described below.

The first three steps are feasibility checks, not preferences. No optimization
level or command-line override can make an infeasible plan selectable.

## Cost comparison

A plan estimate is a vector rather than a single score. Its dimensions can
include latency, reciprocal throughput, code bytes, peak live storage,
allocation, transferred bytes, occupied target resources, energy, variation,
and compilation work. Each fact retains the units, conditions, provenance, and
uncertainty required by the architecture-model design.

The compiler first applies Pareto dominance: plan `A` dominates plan `B` when
`A` is proved no worse in every comparable relevant dimension and strictly
better in at least one. Unknown and incomparable dimensions do not count as
equal or favorable. A dominated plan is not selected while its dominating plan
remains feasible.

When several nondominated plans remain, the active profile supplies ordered
goals. Each goal compares only its own dimension and declared statistic or
bound. The first goal which proves a difference decides. If no goal proves a
difference, the compiler uses a stable canonical plan identity as the final
tie-break. It does not add cycles, bytes, joules, or probabilities into an
unversioned unitless score.

An optimization may use a calibrated model only inside the calibration's
qualified target, board, operating mode, workload shape, toolchain, and
tolerance. When required cost information is absent, the plan remains eligible
only if it is otherwise safe and legal; the missing value supplies no
preference. The compiler chooses conservatively and can explain what fact would
have distinguished the alternatives.

Source-level hard resource classifications and `Prefer` retain the semantics
defined in [resource complexity guarantees](performance.md). A hard
classification participates in feasibility. An applicable source `Prefer`
orders the affected implementation choice before the compiler profile's soft
goals. Compiler flags cannot contradict a hard source requirement.

## Standard profiles

The standard optimization spelling selects a versioned policy profile, not a
promise of a particular pass list. A compiler records the profile and exact
optimization-plan revision in its artifact metadata.

| Profile | Ordered intent | Permitted tradeoff |
| --- | --- | --- |
| `-O0` | faithful diagnostic reference, low compilation work | no optional Topal rewrite or specialization; mandatory lowering, verification, and backend code generation still run |
| `-O1` | compilation work, executable latency, code size | inexpensive conservative transformations; avoid costly search and large growth |
| `-O2` | executable latency, throughput, code size, compilation work | balanced default for an explicitly optimized build |
| `-O3` | executable latency, throughput | greater code growth and compilation work when the model supports the expected benefit |
| `-Os` | code size, executable latency | size wins unless it violates a hard limit; avoid growth without a proved size-neutral speed benefit |
| `-Oz` | code size | minimize deployed code even when that is expected to reduce speed, subject to hard limits |

`-O0` is the compiler's semantic differential reference. The compiler's
invocation default remains `-O0` until the optimized pipeline has the required
conformance coverage. A future revision may make a higher level the default
only as an explicit, versioned policy decision.

The standard profiles do not silently select `native` CPU features. Target
selection and optimization intent are independent.

## Target selection

Without a target option, a compiler which supports the compilation host uses a
recorded generic baseline for the host architecture family and platform. It
does not probe optional CPU features. In the current compiler increment, that
generic baseline is the sole qualified `x86_64-unknown-linux-gnu` target; other
hosts and explicit foreign targets are rejected before lowering until their
profiles are qualified.

The command-line target controls are:

```text
--target <triple-or-profile>
--cpu generic|native|<profile>
--board <profile>
--target-model <path>
```

`--target` selects a qualified generic, specific, or foreign target profile.
`--cpu generic` selects that profile's baseline. `--cpu <profile>` selects a
compatible named processor model. `--cpu native` explicitly requests qualified
build-host detection and is invalid for a foreign target. `--board` composes a
qualified board overlay. `--target-model` supplies the path of an additional
declarative Topal architecture package; the package must validate and its
identity is recorded. Explicit selections which disagree are errors rather
than precedence-based guesses.

Cross-compilation is ordinary explicit target selection. It does not imply
that the result can run on the build host, and it does not authorize execution
of target probes. Runtime feature detection and multiversioning are separate
optimizations: they require an explicit qualified baseline fallback and may be
chosen only when their dispatch and deployment dependencies are available.

## Goal, limit, and pass controls

Profiles can be refined with the following interface:

```text
--optimization-goal <dimension>
--optimization-limit <dimension>=<quantity>
--enable-optimization <stable-id>
--disable-optimization <stable-id>
--only-optimization <stable-id>
--list-optimizations
--explain-optimizations[=<path>]
```

`--optimization-goal` is repeatable. Its occurrence order replaces the
profile's soft-goal order; supported dimensions and quantities are listed by
the compiler. `--optimization-limit` is repeatable and adds a hard constraint
in an explicit unit. Contradictory limits are errors.

Every optional Topal pass has a stable ID. `--enable-optimization` adds a pass
to the profile and `--disable-optimization` removes it. When both mention the
same ID, the later option wins. `--only-optimization` selects the `-O0`
optional-pass set plus the named pass and is mutually exclusive with a
standard optimization level or another `--only-optimization`. Dependencies
required by an enabled pass may run and are shown in the plan. Mandatory
semantic lowering, validation, LLVM verification, and backend correctness
work are not optional passes and cannot be disabled.

An enabled pass still runs only where its semantic and target preconditions
hold. Enabling a pass is not permission to assume missing facts. Unknown pass
IDs, goals, dimensions, quantities, models, targets, CPUs, and boards are
errors. An implementation may report that a known optimization is unavailable
in its revision, but may not silently substitute another one.

`--list-optimizations` is a read-only query which prints each stable ID, a short
description, implementation status, required evidence, and the profiles which
normally enable it. It does not require source input or produce an artifact.

`--explain-optimizations` records the effective target/model identities,
profile, ordered goals, hard limits, enabled pass set, candidates considered,
preconditions, cost evidence, rejections, missing facts, and final decisions.
The default destination is standard error; a path writes a deterministic
machine-readable report. Paths and timestamps are excluded from the canonical
decision content.

## Diagnostics and reproducibility

When missing source facts could materially change a selected plan, the
compiler should emit a configurable optimization remark. Examples include
unknown alignment, aliasing, value range, trip count, workload shape, transfer
volume, and hotness. When target or board information is missing, the message
identifies the architecture-model provider rather than inviting ordinary
source code to assert hardware truth. A remark states the affected
optimization, the conservative decision, the missing fact, and a valid way to
supply or prove it. It is never a language error unless a hard requirement
cannot be met.

Given the same source identities, dependency artifacts, compiler and LLVM
identities, architecture packages, explicit options, and qualified calibration
inputs, plan selection and its explanation are deterministic. Host discovery,
environment variables, profile data, and filesystem order affect selection
only when explicitly admitted and recorded. The artifact records enough of the
effective policy and target identity to reproduce or reject reuse of the plan.

## Deferred policy

This policy does not yet standardize profile-guided input collection,
autotuning budgets, machine-learned policies, runtime dispatch ABIs, or a
portable real-time scheduling proof. Those mechanisms can be added only with
versioned provenance, reproducibility, deployment, and failure rules. The
[optimization research library](compiler-optimizations/README.md) remains a
catalog of candidates, not a promise that a profile implements them.

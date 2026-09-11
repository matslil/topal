# Resource complexity guarantees

Topal distinguishes semantic capabilities from resource complexity guarantees.
A capability promises that an object and its operations behave correctly. A
resource complexity guarantee describes how an implementation's resource use
grows as measures of its arguments or represented values grow. Forgetting the
latter may lose an optimization opportunity, but cannot make an otherwise
correct operation invalid.

Complexity guarantees are nevertheless real guarantees. When a declaration
requires one, only implementation evidence satisfying that bound is applicable.
Optional optimization comes from ordered specialization with a semantically
valid fallback, not from treating a failed guarantee as if it had matched.

## Fundamental guarantees

The selected language version supplies three fundamental constructions:

```text
OExec Expression
OAlloc Expression
NoAlloc
```

`OExec ( E )` is an asymptotic upper bound on abstract execution work.
`OAlloc ( E )` is an asymptotic upper bound on total dynamically allocated
storage. Each directly accepts an Ordo expression, so there is no additional
`Ordo` wrapper.

The expressions describe dependence on static measures of the classified
object's inputs or represented values. They do not express seconds, processor
cycles, allocator latency, or a concrete number of bytes. A function may, for
example, publish:

```topal
sort is fn (
  values : C : Sortable
)
  guarantees (
    OExec ( (values size) ^ 2 )
    and OAlloc ( values size )
  )
-> C
```

This says that execution work grows no faster than the square of the input
size, while total allocation grows no faster than the input size. Constant
factors and lower-order terms are deliberately absent.

Several arguments may contribute independent measures:

```topal
combine is fn (
  left : ( L : Counted ),
  right : ( R : Counted )
)
  guarantees (
    OExec ( (left size) * (right size) )
    and OAlloc ( (left size) + (right size) )
  )
-> Combined
```

The measure operations available to an expression must be static, pure, total,
and visible from the complete declaration. A measure is not an implicit field
name: `values size` is an ordinary applicable static observation whose result
can participate in the expression.

## Exact absence of allocation

`NoAlloc` is an exact guarantee that the classified implementation performs no
dynamic allocation for any valid input. It is deliberately distinct from an
asymptotic allocation class.

`OAlloc ( 1 )` permits a bounded amount of allocation independent of input
size. `OAlloc ( 0 )` retains its ordinary asymptotic meaning: allocation is zero
beyond some input threshold, but may occur for a finite prefix of inputs.
Neither implies `NoAlloc`. `NoAlloc` implies both bounds.

Future language versions may distinguish heap, arena, task, or other allocation
regions with additional exact guarantees. The initial vocabulary treats
dynamic allocation as one resource dimension and does not reinterpret
`NoAlloc` as merely constant allocation.

## Classification and composition

Complexity guarantees can classify functions, container representations, and
other static implementation evidence for which their measures are defined.
They refine the admitted implementations in the same broad manner that a
constraint refines admitted values, but they neither change the represented
value nor supply an implementation.

They compose with semantic capabilities using ordinary classifier conjunction.
A mixed conjunction retains both kinds of evidence:

```topal
RandomAccess is Indexed and
  ( Indexed get : OExec ( 1 ) )
```

`Indexed get` statically projects the exact ordinary operation identity assigned
to the `get` role by canonical `Indexed` evidence for the same classified
subject. `Indexed` promises that positional access exists and behaves correctly,
while `OExec ( 1 )` promises constant asymptotic execution for that existing
implementation. Projecting only `Indexed` forgets the performance evidence
without losing semantic correctness. Requiring the complete `RandomAccess`
combination requires both.

Parameterized static functions may give descriptive names to useful bounds:

```topal
QuadraticExecution is fn static (
  N : Nat
) -> PerformanceGuarantee
  OExec ( N ^ 2 )
```

Such names combine or parameterize language-defined guarantees. As with
capabilities, ordinary libraries cannot invent an atomic resource dimension
whose meaning or verification rules the selected language version does not
define.

## Ordered specialization

Missing complexity evidence does not make the underlying semantic operation
incorrect. Code can offer a performance-specialized declaration before a
general declaration:

```topal
find is fn (
  values : C : RandomAccess,
  wanted : Object
) -> Optional Nat
  optimized-body

find is fn (
  values : C : Indexed,
  wanted : Object
) -> Optional Nat
  general-body
```

Normal ordered matching selects the first applicable declaration. When
`RandomAccess` evidence is unavailable, the first header does not match and the
semantically sufficient `Indexed` implementation remains applicable. If no
fallback is declared, the complexity guarantee is a mandatory part of that
call's contract rather than a hint which the compiler may silently ignore.

## Call-site requirements and preferences

A call may classify the function selected from an overload set with a resource
guarantee. The classification is a hard requirement:

```topal
found is mycontainer
  ( search : OExec ( size mycontainer ) )
  my-key
```

The `search` implementation must be applicable to the two ordinary operands and
must carry execution evidence no worse than the stated class. If none does,
the call is invalid. When several implementations satisfy the requirement,
ordinary source order still decides unless the call also supplies a preference.

`Prefer` turns one or more resource guarantees into soft, ordered selection
goals:

```topal
found is mycontainer
  (
    search
      : Prefer (
          OAlloc ( size mycontainer ),
          OExec ( (size mycontainer) ^ 2 )
        )
  )
  my-key
```

The product passed to `Prefer` is lexicographic. Selection first prefers
applicable implementations whose allocation evidence satisfies the first
goal. Among implementations in the best allocation class, it then considers
the execution goal. A strictly tighter proven Ordo class wins within one goal;
equivalent or incomparable classes do not reorder declarations. Source order
is the final tie-breaker.

Unlike direct classification, failing a preferred bound never makes an
implementation inapplicable. If no candidate proves a preference, that
preference imposes no ordering and selection continues with the next goal or
source order. Missing evidence is not fabricated and does not satisfy a
preferred class.

Hard and soft classifications may be combined:

```topal
found is mycontainer
  (
    search
      : NoAlloc
        and Prefer ( OExec ( size mycontainer ) )
  )
  my-key
```

Every allocating implementation is rejected. Among the remaining
implementations, the best applicable execution class satisfying the preference
is selected before source order. `Prefer ( NoAlloc )` would instead make exact
zero allocation desirable while retaining allocating fallbacks.

`Prefer` is a selection construction, not evidence and not a weakened
guarantee. It is meaningful only while resolving an implementation-bearing
static object. Ordinary `and` remains order-independent; only the product
inside `Prefer` records priority order. Without an explicit `Prefer`, resource
complexity never silently reorders overload declarations.

Optimization based on unverified foreign or opaque metadata must remain
semantics-preserving when that metadata is false. Such evidence may choose a
faster equivalent implementation, but cannot authorize memory-safety,
termination, effect, or capability assumptions.

## Derivation

Visible typed intermediate code retains symbolic complexity expressions and
the measures they reference. A compiler may derive and simplify guarantees by
ordinary asymptotic composition:

- sequential work and allocation add, with dominated terms simplified;
- alternatives use a conservative upper class;
- bounded repetition combines its iteration class with the body class;
- a call substitutes the caller's argument measures into the callee's
  expression;
- specialization substitutes concrete evidence and may produce a tighter
  class; and
- erased dynamic alternatives retain a conservative common upper class when
  one exists, otherwise their complexity is unknown.

Inference which cannot establish a requested class leaves that guarantee
unproved. An opaque or foreign implementation must publish checked, trusted, or
conservatively enforced evidence before a caller may rely on its bound.

Execution work is intentionally abstract. Mapping it to elapsed-time or
real-time deadlines is target-specific and requires platform evidence. In
`v0.1`, total allocation is likewise distinct from peak live memory. Revision
`v0.2` adds the portable space dimensions below without changing `OAlloc`.

## Revision `v0.2` resource bounds

`v0.2` supplies `ResourceBound ( dimension is D, scope is S, bound is B )`.
Portable dimensions are `Work`, `Span`, `AllocationTotal`, `PeakLive`,
`Retained`, `Stack`, `QueueEntries`, `TransferCount`, and `CodeSize`. `OExec`
and `OAlloc` remain asymptotic shorthands for work and total allocation;
`NoAlloc` remains exact.

Sequential work, total allocation, and dependent span add. Alternatives take a
conservative maximum. Independent span takes a maximum. Peak and retained
storage use lifetime overlap rather than total allocation. Queue entries and
transfers compose for the named endpoint or boundary. Stack and code size are
facts about a concrete lowering. An unknown bound is distinct from infinity
and never satisfies a hard requirement.

`ProgressClass` is the closed order `MayBlock`, `ObstructionFree`, `LockFree`,
and `WaitFree ( maximum-own-steps is B )`. It classifies a complete selected
interaction implementation. A programmer may require or prefer a class but
cannot create its evidence. Progress is conditional on recorded scheduling and
hardware-completion assumptions and does not imply an elapsed-time bound.

All function-level requirements use the pre-arrow `guarantees` clause shown at
the start of this document. `Prefer` remains a soft lexicographic choice. A
diagnostic for a failed hard bound names the dimension, scope, derived or
unknown value, assumptions, and candidate implementations.

## Specialization and implementation plans

`Specialized ( static-inputs is S )` requires that the final compiled
implementation substitute `S` and contain no residual tag, dictionary, closure,
or dispatch attributable solely to those inputs. It does not require unrelated
dynamic work to be inlined. Only a compiler or checked precompiled artifact can
establish this code-shape evidence.

The compiler owns a typed implementation-plan IR containing fusion,
materialization, iteration, tiling, buffer reuse, channel choice, layout,
conversion, transfer, and dependency decisions with their evidence. Programs
cannot read or edit it. Tools may request a stable diagnostic projection, but
editing that projection cannot feed a plan or proof back into compilation.

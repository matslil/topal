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
) -> C
  : OExec ( (values size) ^ 2 )
    and OAlloc ( values size )
```

This says that execution work grows no faster than the square of the input
size, while total allocation grows no faster than the input size. Constant
factors and lower-order terms are deliberately absent.

Several arguments may contribute independent measures:

```topal
combine is fn (
  left : ( L : Counted ),
  right : ( R : Counted )
) -> Combined
  : OExec ( (left size) * (right size) )
    and OAlloc ( (left size) + (right size) )
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

```text
RandomAccess =
  Indexed
  and constant execution evidence for Indexed's access operation
```

The exact operation-association surface syntax remains part of the general
classification grammar design. Its semantics are fixed: `Indexed` promises
that positional access exists and behaves correctly, while the associated
`OExec ( 1 )` evidence promises constant asymptotic execution for that existing
operation. Projecting only `Indexed` forgets the performance evidence without
losing semantic correctness. Requiring the complete `RandomAccess` combination
requires both.

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
real-time deadlines is target-specific and requires platform evidence outside
this initial model. Total allocation is likewise distinct from peak live
memory; a future resource dimension may describe space complexity without
changing `OAlloc`.

# Carried-state and one-step storage proofs

This implements the range/allocation part of `TK-DEC-282` Stage 2 under
`TOPAL-SYSTEMS-TAIL-REUSE-001`. It does not itself qualify a checkpoint,
native stack, lifetime transfer or kernel invocation.

## Input domain and invariants

The first x86-64 analysis uses the existing native-word fact domain: initial
Nat inputs are 0..M, where M = 2^64 - 1. An Int analysis uses -M..M, and Boolean
state retains its ordinary two values. An executable caller must bind the
corresponding actual interface and input checks. These are qualification facts,
not finite-width source arithmetic. Source values may grow beyond those bounds.

Rebuild the source-tail plan from the checked program. For each tail edge,
recover its exact positive measure decrement d. Resolve ordinary immutable
bindings by storage identity and derive linear expressions through scalar
arithmetic and closed helpers. A helper's result is linear only when every
possible result has the same expression. Reads, writes and all evaluated
arguments remain accounted for separately; symbolic reasoning performs no
source operation and does not authorize removing one.

For a carried numeric parameter x, a recognized update x' = x + c can preserve
two potentials:

```text
 x + wp*m <= Rmax + wp*M
-x + wn*m <= -Rmin + wn*M
```

Choose wp at least ceil(c/d) for every positive affine drift c, and wn at least
ceil(-c/d) for every negative drift. The baseline [Rmin,Rmax] contains the
initial domain and all directly established constant resets. The measure m
never exceeds M and decreases on every edge. A non-affine reset must be proven
inside the baseline, rather than merely inside the wider global interval.
That distinction prevents a reset from invalidating the potential before a
later affine update. Unproved updates fail before publication.

The resulting all-step interval is [Rmin-wn*M, Rmax+wp*M]. On the recursive
branch, m >= b+1 further tightens it through the same potentials. The source
measure guard, not a machine iteration counter, proves progress. In particular,
the byte traversal's remaining+offset invariant establishes a 65-bit offset
envelope from native-word initial inputs. Repeatedly widening an interval by one
does not establish that invariant.

## Complete source work for one step

Reuse the reviewed numeric/region allocation analysis for ordinary expressions,
helpers, arithmetic and byte operations. A control-flow walker handles tail
positions, evaluates each argument exactly once in source order, checks the
successor state and charges its complete source evaluation. It does not follow
the recursive edge or emit a replacement function. Prefix bindings, the guard,
both reachable actions and the terminal step enter the bound. Branch maxima
account for exclusive actions; all potentially tested ordered operands count.

Subtraction is enabled only for this separate analysis. The bound is reviewed
against the exact shared runtime digest: finite subtraction negates/copies the
right operand and performs finite addition, with at most two integer allocations.
The existing conservative eight-allocation arithmetic charge, including extra
limbs/alignment, covers both. Existing numeric/region profiles keep their old
admission behavior and certificates. The 256-bit numeric-runtime proof ceiling
is unchanged.

Charge every numeric transfer root separately, including aliases and constants,
and both simultaneously owned banks. The private bound includes the header,
control, invariant view where present, banks, initial bridge values and complete
one-step work. Caller-owned immutable input-frame size is recorded separately;
it is not source scratch that a checkpoint may erase. Unsupported or overflowing
proofs and a total above 65536 bytes are diagnostics. A computed allocation
envelope remains conditional on correctly implemented retirement and reuse.

An initial Nat magnitude costs at most 24 aligned bytes. Initial signed Int
construction may allocate that magnitude and a second sign-adjusted copy, so
charge 48 bytes. For the complete -M..M fact domain the conservative caller
frame reserves a magnitude word and a sign word per Int; Nat/Boolean facts each
reserve one word. A future narrower signed-word interface must separately bind
its input representation rather than assuming that the larger fact domain fits
one word.

## Identity and subsequent evidence

Count actual source call multiplicities through the closed helper DAG, including
arguments and branch costs. Function identity count or one-step depth is not
that bound. Fresh frames and fresh region loans are counted independently from
simultaneous live storage. Before mutation, a native caller must admit the
runtime-derived maximum step count against the remaining nonwrapping identity
budget. A constant-memory traversal cannot reset authority history to reuse it.

A fresh invocation has live root identity 1 and next-frame/view counters 2.
Issuing a label first checks the successor counter, so the maximum counter
value cannot itself be issued. With per-step bound p and current next n, admit
at most floor((M-n)/p) complete steps, including the terminal step. Existing
histories use their actual next counters; the fresh-invocation ceiling is not
permission to reset one. This also reserves the initial root and avoids an
off-by-one admission when p divides the remaining counter space exactly.

Range/allocation metadata binds the source plan, checked graph, initial domain
and reviewed runtime inputs. It grants neither source origin for external IR
nor executable qualification. Native transfer must still preserve every live
root, revoke retired dependencies, erase actual storage, satisfy the 4096-byte
stack bound and reach trusted fatal cleanup after admitted persistent effects.
Independent guarded native/physical evidence and real owner integration remain
the rest of Stage 2 and the subsequent stages.

# Fundamental standard-library completion plan

This plan completes the ordinary Topal library around the fundamental types.
It supplements rather than wraps primitive operators. Parsing, formatting,
display, locale policy, platform facilities, and representation-specific
conveniences are outside this series.

The completed fundamental library is published directly through one flat
`std` namespace and one ordinary `library/std/module.t` source module. Future algorithm
packages use separate namespaces; they do not enlarge or subdivide `std`.

## API rules

- Use the narrowest capability supplying every operation an algorithm needs;
  never specialize because a tool temporarily lacks generic support.
- Preserve exact related type variables through inputs and results, including
  `Optional T`, `Result (T, E)`, `Range T`, and `Sequence T`.
- Use concrete overloads only when semantics inherently belong to that domain.
- Prefer established short names: `min`, `max`, `min-max`, `gcd`, `lcm`,
  `map`, `zip`, and `scan`. Do not abbreviate merely for brevity.
- Partial operations return `Optional` or `Result`; they do not invent a
  default, trap, or silently clamp.
- Do not alias an existing primitive without an additional semantic contract.

## Native GitHub stack

The series uses GitHub's stacked-pull-request feature through the official
`gh stack` extension. Every layer is a focused branch above its predecessor,
is submitted as one server-linked stack, and is checked against the `main`
trunk protections. Cascading updates use `gh stack rebase` and
`gh stack push` or `submit`; an ordinary inferred PR chain is insufficient.

## Layers

1. **Roadmap and conformance matrix.** Record scope, API admission rules,
   existing primitives, intended additions, and deliberate omissions.
2. **Structural and higher-order generics.** Support related type variables,
   structural classifiers, higher-order inference, exact substitution,
   diagnostics, and GEIR representation.
3. **Ordering.** Add generic `min`, `max`, `min-max`, safe bound handling,
   extrema over finite folds, and `*-by` forms under `TotalOrder`.
4. **Optional.** Add generic queries, `map`, `chain`, `filter`, fallback,
   `zip`, `flatten`, caller-error Result conversion, traversal, and aggregation;
   forced extraction remains absent.
5. **Result and Error.** Add mapping, chaining, recovery, fallback, zip,
   flatten, transpose, traversal, and aggregation while preserving code
   vocabulary, domain, detail, cause, and provenance.
6. **Exact numbers.** Add precisely constrained sign, distance, sums,
   products, averages, `gcd`, `lcm`, divisibility, parity, reciprocal, fraction
   utilities, and checked conversions without a broad `Numeric` capability.
7. **Ranges and indexes.** Generalize `Range T` under `TotalOrder`; add bound
   observation, emptiness, intersection, hull, overlap, adjacency, splitting,
   and finite-index conversion. Traversal also requires discrete successor and
   termination evidence.
8. **Character, String, and Unicode.** Add classification, prefix/suffix,
   search, split/join, trim, replacement, repetition, checked character
   regions, normalization, and caseless forms. Bytes remain distinct from text.
9. **Finite folds and sequences.** Add quantifiers, search, reduction, scans,
   flat-map, partitioning, chunks, windows, intersperse, grouping,
   deduplication, zipping, and evidence-constrained sorting without duplicating
   existing primitive map, select, or fold operations.
10. **Generators and lazy algorithms.** Add lazy transformation, selection,
    flat-map, take/drop forms, scan, zip, enumeration, chaining, repetition,
    and replayable cycling while retaining final results and close behavior.
11. **Array.** Add checked generic access, replacement, transformation,
    slicing, concatenation, reversal, traversal, and evidence-based sorting.
12. **Map.** Add lookup, containment, insertion, replacement, removal,
    update, explicit-policy merge, traversal, mapping, filtering, and derived
    equality only when its evidence exists.
13. **Set and Bag.** Add set algebra, subset/disjointness relations, and Bag
    multiplicity arithmetic without exposing hash or tree representation.
14. **Laws and final audit.** Add executable laws, differential tool
    observations, optimization identities, complexity contracts, and an
    explicit implemented-or-deliberately-omitted matrix.

## Per-layer completion

Each layer propagates approved design through specification, tool requirements,
tests, implementation, traceability, and commented shared Topal examples.
Interpreter, scripted debugger, LSP, linter, and compiler/GEIR paths are updated
where applicable. Every layer passes formatting, strict workspace linting, the
complete workspace test suite, diff validation, and GitHub trunk-level stack
checks before the next layer is submitted.

The completed disposition of every layer is recorded in
[`fundamental-standard-library-matrix.md`](fundamental-standard-library-matrix.md).
Entries recorded there as deliberate omissions are terminal for this series;
they require a separately admitted contract before later publication.

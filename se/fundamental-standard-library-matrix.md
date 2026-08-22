# Fundamental standard-library completion matrix

This matrix closes the completion series defined in
[`fundamental-standard-library-completion.md`](fundamental-standard-library-completion.md).
An implemented entry is either ordinary source in `library/std/module.t` or an
irreducible representation-dependent root operation. A deliberate omission is
not a placeholder API: adding it later requires its own admitted contract and
must not silently enlarge the fundamental `std` namespace.

| Layer | Implemented | Deliberately omitted from fundamental `std` |
| --- | --- | --- |
| Roadmap and conformance | flat namespace, admission rules, cross-tool application, this terminal matrix | none |
| Structural and higher-order generics | exact Optional, Result, List, Range, product, and named-function result substitution; bound generic type values | inference of an anonymous function's result before its first invocation |
| Ordering | `min`, `max`, `min-max` with left-biased ties | `*-by` and finite extrema until a distinct key-order/effect contract is admitted |
| Optional | queries, map, chain, filter, value and Optional fallback, zip, flatten | forced extraction; absence-to-error conversion until ordinary Error construction can retain caller vocabulary and provenance |
| Result and Error | queries, success/error map, chain, recovery, fallback, zip, flatten | collection traversal/aggregation until early-exit folds retain complete Error evidence |
| Exact numbers | sign, distance, exact List sum/product, gcd, parity, divisibility, reciprocal | parsing/formatting; fraction projection and checked conversions already supplied by exact constructors; `lcm` until nonzero quotient evidence is expressible in source |
| Ranges and indexes | bounds, intersection, hull, overlap, Int adjacency; checked `array-at?` | generic traversal without discrete-successor and termination evidence |
| Character, String, Unicode | NFC/NFD, canonical/caseless equality, search, trim, replacement, repetition | locale policy, bytes, parsing, formatting, and aliases of existing case/character primitives |
| Finite folds and sequences | quantifiers, predicate count, find, filter-map, flat-map, exact sum/product | aliases of primitive map/select/fold and algorithms whose promised complexity needs a stronger sequence builder |
| Generators and lazy algorithms | replay-free `count-from`; primitive iterate, unfold, take-while, collect, and foreach remain directly available | wrappers that add no contract; replayable cycle without replayability evidence; transforms that would erase final or close results |
| Array | construction, count, emptiness, checked `array-at?` | aliases of List algorithms and shape-changing transforms that require existential result-size support |
| Map | collision-policy construction, count, emptiness, exact `map-lookup` | iteration without explicit order; update/merge until persistent association builders are ordinary source capabilities |
| Set and Bag | construction, count, emptiness, `set-contains?`, `bag-multiplicity` | iteration without explicit order; algebra that would expose or depend on a representation-specific builder |
| Laws and optimization | executable ordering and normalization laws, shared differential application, structural GEIR identity contract | mandatory compiler substitutions; every substitution remains optional |

The omissions preserve the architecture boundary: the fundamental library does
not acquire hidden builders, reflection, ordering, error construction, or
replay authority merely to make a checklist longer. Future independent
algorithm packages may add policy-bearing operations in their own namespaces.

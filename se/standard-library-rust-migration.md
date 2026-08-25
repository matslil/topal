# Rust-to-Topal standard-library migration

## Purpose and boundary

This inventory implements `TOPAL-LIB-SOURCE-001` and the source-first boundary
in `se/standard-library-bootstrap.md`. Portable observable library semantics
belong in ordinary `.t` files. Rust remains appropriate only for parsing and
executing the core language, representing fundamental values, checked access
to host memory, and explicit operating-system or device adapters that Topal
cannot implement portably.

The classification is semantic rather than name-based. A checked primitive
which exposes a fundamental value operation may remain in the evaluator. An
algorithm does not become a core primitive merely because a Topal wrapper
calls it by an unqualified hidden name.

## Complete migration inventory

### Text, parsing, and pattern algorithms

Move the portable implementations behind these evaluator operations to
`std.text`, `std.parse`, or `std.pattern`:

- `string-starts-with`, `string-ends-with`, `string-contains`,
  `string-count-exact`, `string-find-all`, `string-split-exact`,
  `string-contains-any`, `string-lines`, `string-words`, and `string-join`;
- `string-trim`, `string-replace-all`, and `string-repeat`;
- `string-glob-matches` and `string-regex-contains`;
- `string-parse-int`, `string-signed-integers`, `string-unsigned-integers`,
  `string-decimal-digits`, `string-integer-rows`,
  `string-vertical-integers`, `string-integer-pairs`, and
  `string-integer-triples`;
- `character-list-string` and `int-decimal-string`.

UTF-8 decoding, Unicode normalization, character boundary selection, and the
primitive conversion between one `Character` and its textual representation
remain core representation operations. A future regular-expression engine may
use a separately specified native acceleration backend, but matching semantics
and the portable fallback remain Topal-defined.

### Finite sequence and range algorithms

Move these operations to `std.sequence` or the extended range library:

- `list-index-of`, `list-last-index-of`, `list-rotate-left`,
  `list-rotate-right`, `list-chunks`, `list-windows`, `list-enumerate`,
  `list-group-runs`, `list-zip-shortest`, and `list-transpose-shortest`;
- `range-integers` and `range-coalesce-int`;
- the library-facing behavior of `reverse`, `stable-sort`,
  `stable-sort-descending`, `ordered-binary-search`, `ordered-merge`,
  `ordered-smallest`, and `ordered-nth`.

Fundamental immutable `List` construction/decomposition, entry count, checked
index/range selection, append/concatenation, fold, equality, and comparison
remain core operations. They are the substrate on which the Topal algorithms
are implemented.

### Combinatorics

Move `list-permutations`, `list-combinations`, `list-subsets`, and
`list-cartesian-product` to `std.combinatorics`. Factorial and subset count are
already implemented in Topal and require no migration.

### Exact statistics

Move all implementations dispatched by `apply_statistics` to
`std.statistics`: median, modes, histogram, population and sample variance,
quantile, covariance, summary construction, summary addition/merge, summary
mean, and summary variance. Exact `Int`/`Rational` arithmetic remains core.

### Graph algorithms

Move all implementations dispatched by `apply_graph_algorithm` to `std.graph`:
breadth-first and depth-first traversal, unweighted and weighted shortest
paths, topological sorting, weak components, adjacency construction, and path
reconstruction. Also move the described-graph parsing and path-count helpers
currently dispatched by `apply_planning_algorithm`.

### Geometry, machine, and packing algorithms

Move all of `apply_geometry_algorithm` and its union-find, distance,
rectangle, and containment helpers to `std.geometry`. Move machine-manual
parsing and minimum-press search to `std.machine`. Move shape normalization,
orientation generation, region-fit search, and described packing to
`std.packing`.

These are ordinary portable algorithms and are not valid evaluator primitives.

### Transfer semantics currently duplicated in Rust

The portable models in `src/topal-transfer` are migration or deletion targets:

- compatibility negotiation;
- database row/schema policy and file-store naming policy;
- device ownership state, message framing, and firewall policy;
- host authority/replay policy, I2C transaction policy, and network-family
  policy;
- operation completion/admission policy, regions/views, store consistency,
  and transport close/backpressure behavior.

Their authoritative replacements are the modules below `library/std` and
their Topal tests. Rust may remain only for explicit native adapters and their
safe translation of OS results into the Topal contracts. `native.rs` is
therefore reduced to platform capability declarations and later concrete
adapters; it must not carry a second portable semantic implementation.

### Advent-of-Code-specific evaluator operations

The descriptive geometry, graph, machine, and packing operations were added to
support examples but are standard-library algorithms or application parsing,
not language features. The examples shall compose the migrated Topal modules;
no puzzle-specific solver remains in the evaluator.

## Explicitly retained Rust boundary

The following are not migration targets:

- lexing, parsing, diagnostics, type/classifier checking, lexical scope,
  function/generator/task execution, tracing, and module loading;
- fundamental value representation and construction for numeric, product,
  sum, optional/result, range, collection, layout, and serialization values;
- exact primitive arithmetic, comparison, equality, Unicode tables and
  normalization, checked memory/layout access, and generator mechanics;
- command-line file loading and test-process orchestration;
- explicit OS, network, filesystem, database-driver, and device-controller
  adapters whose effects cannot be expressed by portable Topal alone.

## Execution order

1. Implement reusable Topal sequence and textual building blocks using only
   retained core operations.
2. Rebuild ordered, combinatorial, and statistical algorithms on those blocks.
3. Rebuild graph algorithms, followed by geometry, machine, and packing.
4. Redirect every public module and example to the Topal implementation and
   add direct Topal conformance coverage for edge cases previously tested only
   through Rust.
5. Delete each evaluator dispatch name and Rust helper immediately after its
   final Topal caller disappears.
6. Delete portable `topal-transfer` models after their Topal tests provide
   equivalent evidence; retain only explicit native adapter boundaries.
7. Verify that no hidden algorithm operation remains in the root operation
   catalog and that every `std` public behavior resolves to `.t` source.

The migration is complete only when searching Rust sources for every operation
listed above finds neither dispatch nor an algorithm implementation, while all
Topal, debugger, LSP, linter, documentation, and resource tests pass.

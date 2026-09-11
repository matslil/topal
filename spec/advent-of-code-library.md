# Advent of Code companion-library conformance

The `advent-of-code` library contains reusable implementation policy for the
repository's Advent of Code applications. It is an ordinary, explicitly
selected Topal library and is not part of the language standard library. Its
public operations MAY reflect the input representations and problem rules of
the applications that consume it.

### TOPAL-LIB-GEOMETRY-001 — Exact finite puzzle geometry

The `advent-of-code geometry` namespace SHALL provide deterministic
nearest-first component clustering for exact three-dimensional integer points
and inclusive axis-aligned rectangle maximization for exact two-dimensional
integer points. The contained-rectangle operation SHALL interpret its vertices
in order as a closed orthogonal polygon and reject rectangles crossed through
their interior by a polygon edge. Equal-distance point pairs SHALL be ordered
by their source indexes.

### TOPAL-LIB-PLANNING-001 — Finite exact puzzle planning

The `advent-of-code machine` namespace SHALL minimize presses for finite
binary-indicator and nonnegative additive-counter machines. The
`advent-of-code graph` namespace SHALL count routes from the repository's
described finite directed-acyclic-graph format, optionally requiring a finite
set of intermediate nodes. The `advent-of-code packing` namespace SHALL decide
exact rectangular packing of requested free polyominoes, considering rotations
and reflections without overlap. These operations SHALL return exact integer
results and SHALL be deterministic.

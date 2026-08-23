# Standard-library algorithms

Algorithms live below `std` in namespaces that describe the information they
require, rather than the host data structure used to execute them. The
fundamental namespace retains only mechanisms needed to express those
algorithms without exposing storage representation.

`std sequence` contains operations that do not require entry ordering: bounded
prefixes and suffixes, splitting, predicate retention, duplicate removal,
indexed equality search, rotations, chunks, windows, enumeration, adjacent-run
grouping, and shortest zip. `std ordered` adds stable Int and Rational sorting,
binary search, merge, partial and nth-order selection, and insertion boundaries.
`std pattern` distinguishes
consecutive exact matching from ordered subsequence matching.

More specialized namespaces keep their policies visible:

- `std text` applies the selected Unicode context and does not silently add
  locale or encoded-byte policy;
- `std graph` operates on explicit finite nodes and directed edges, including
  cycles, without choosing an application-specific graph representation;
- `std combinatorics` provides exact counts and constructions, with operations
  named for whether entries are distinct or repeated; and
- `std statistics` uses exact numeric results and represents undefined results,
  such as the mean of an empty sample, explicitly.

The first revision intentionally favors small composable operations. More
specialized search structures, regular expressions, graph weights, sampling,
and approximate numeric methods belong in later namespaces with their policy
and complexity contracts stated explicitly.

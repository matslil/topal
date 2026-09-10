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
`std pattern` distinguishes consecutive exact matching from ordered subsequence
matching and provides overlapping search, alternative patterns, and explicit
whole-text `*`/`?` glob policy. Its design-0 regular expressions search for a
matching substring unless anchored. They operate on Unicode scalar values and
provide literals, `.`, bracket classes and ranges, grouping, alternation,
repetition with `?`, `*`, `+`, `{n}`, `{n,}`, or `{n,m}`, and whole-text `^` and
`$` anchors. Dot excludes line feed. Bracket classes use `[^...]` for
complement; `\d`, `\s`, and `\w` select Unicode decimal digits, whitespace,
and word characters, with `\D`, `\S`, and `\W` as their complements. A
backslash quotes a metacharacter.

Ordinary parentheses both group and capture; `(?:...)` groups without
capturing. `std pattern regex captures` reports whether a match exists and, when it does,
returns entry zero for the complete match followed by captures in opening-
parenthesis order. Each entry separately reports participation, so an
unmatched optional group differs from a group that captured an empty String.
Selection is leftmost-longest; a group repeated more than once retains its
last participation.

Backreferences, look-around, conditionals, inline mode flags, and other
backtracking-only constructs are deliberately absent. Malformed patterns are
rejected rather than treated as a non-match. Matching uses a tagged finite-
automaton strategy, so a compiled pattern scales linearly with the searched
text and cannot trigger catastrophic backtracking.

More specialized namespaces keep their policies visible:

- `std text` applies the selected Unicode context to normalization, lines,
  whitespace words, and joining without silently adding locale or encoded-byte policy;
- `std parse` handles strict ASCII decimal conversion, integer extraction,
  decimal digits, and explicit Character materialization without locale policy;
- `std graph` operates on explicit finite String nodes and directed edges,
  including deterministic breadth/depth traversal, minimum-edge and
  nonnegative exact-weight paths, topological ordering, and weak components,
  without choosing an application-specific graph representation;
- `std combinatorics` provides exact counts, positional permutations,
  combinations, subsets, and Cartesian products; repeated equal entries remain
  distinct positions rather than being silently deduplicated; and
- `std statistics` provides exact means, medians, modes, histograms, quantiles,
  population/sample variance, population covariance, and mergeable count/sum/
  square-sum summaries. Undefined results are represented explicitly.

The first revision intentionally favors small composable operations. More
specialized search structures, negative graph weights, approximate sampling,
and approximate numeric methods belong in later namespaces with their policy
and complexity contracts stated explicitly.

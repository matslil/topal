# Advent of Code 2025

This directory contains one Topal application for each computational puzzle in
Advent of Code 2025. The event ended after day 12 part 1; day 12 part 2 awarded
the final star without another computational puzzle, so there are 23 programs.

Run a program with an input file:

```sh
cargo run -p topal-interpreter --bin topal -- \
  --input inputs/day01.input examples/advent-of-code/2025/day01-part1.t
```

Each application defines `solve : fn (String) -> Int`. The explicit input mode
loads the application, passes the complete UTF-8 file to `solve`, and prints
only its result. Put personal puzzle inputs in `examples/advent-of-code/2025/inputs/`;
that directory is ignored because Advent of Code asks participants not to
redistribute inputs.

The committed `tests/advent-of-code/2025/` data is independently constructed
and intentionally small. Run every example against its matching fixture with:

```sh
cargo test -p topal-interpreter --test cli \
  every_advent_of_code_2025_solver_matches_its_topal_test_file
```

The fixtures test the same input shape and algorithmic edge as the puzzle, but
they are not copies of puzzle statements or personal challenge inputs.

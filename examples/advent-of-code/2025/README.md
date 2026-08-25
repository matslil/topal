# Advent of Code 2025

This directory contains one Topal application for each computational puzzle in
Advent of Code 2025. The event ended after day 12 part 1; day 12 part 2 awarded
the final star without another computational puzzle, so there are 23 programs.

Run a program with an input file:

```sh
target/debug/topal examples/advent-of-code/2025/day01-part1.t \
  examples/advent-of-code/2025/day01-part1.input
```

When `topal` is installed on `PATH`, the executable hashbang permits the same
application-oriented invocation:

```sh
examples/advent-of-code/2025/day01-part1.t \
  examples/advent-of-code/2025/day01-part1.input
```

Each application defines `solve : fn (String) -> Int`. Its sole user argument
names the input file; the launcher passes the complete UTF-8 contents to
`solve` and prints only its result. The independently constructed committed
`.input` fixture is kept beside each application. Put personal puzzle inputs
in `examples/advent-of-code/2025/inputs/`; that directory is ignored because
Advent of Code asks participants not to redistribute inputs.

The committed inputs are intentionally small. Each `.t` file under
`tests/advent-of-code/2025/` is a Topal application-test manifest, while only
the expected `.output` value remains with the tester.
Run every example against its matching fixture with:

```sh
topal test tests/advent-of-code/2025
```

The fixtures test the same input shape and algorithmic edge as the puzzle, but
they are not copies of puzzle statements or personal challenge inputs.

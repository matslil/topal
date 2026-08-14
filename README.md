# Topal

Topal is an experimental, general-purpose programming language built from a
small set of recursively composable concepts. Its source language is pure and
functional, while implementations may use mutation, in-place updates, and
parallel execution when doing so preserves the program's meaning.

## Why Topal?

Topal explores what a language can provide when safety, composition, explicit
effects, deterministic parallelism, and machine-checkable contracts are parts
of one coherent model. Its goals include:

- immutable source-level values with safe implementation-level optimization;
- total functions and explicit errors rather than exceptions;
- constraints, capabilities, dependent relationships, and proof evidence;
- deterministic concurrency with compile-time prevention of data races and
  deadlocks;
- explicit layouts and access rules for systems and hardware programming; and
- language semantics precise enough to support independent, interoperable
  tools.

Topal remains an experimental language project with no compiler or stable
public release. Its `v0.1` (`design-0`) core is supported as the repository's
standard-library development and interoperability baseline; see the
[bootstrap contract](se/standard-library-bootstrap.md).

## Repository layout

- [`docs/`](docs/) contains the human-readable language design and is the
  authority for design intent.
- [`decisions.md`](decisions.md) records settled fundamental design decisions.
- [`FUTURE.md`](FUTURE.md) records deliberately deferred work.
- [`se/`](se/) contains system-engineering goals, requirements, validation, and
  traceability information.
- [`spec/`](spec/) contains the normative formal language specifications.
- [`agents/`](agents/) describes the agent roles used to evolve and verify the
  project.
- `src/` contains language tools, shared libraries, and their tests.

Repository-wide agent instructions live in [`AGENTS.md`](AGENTS.md). Each implemented
tool will carry its tool-specific requirements in an `se-requirements.md` file
in that tool's directory.

## Build

Build and test the workspace with Rust 1.97 or newer:

```console
cargo build --workspace
cargo test --workspace --all-targets
```

## Use

The `topal` binary defaults to script execution from a file or standard input:

```console
cargo run -p topal-interpreter -- program.topal
cargo run -p topal-interpreter < program.topal
```

Use `--interactive` for a persistent exploratory session and `--test` for
script execution with stable JSON Lines decision traces on standard error. The
implemented subset and mode contracts are recorded in
[`src/topal-interpreter/se-requirements.md`](src/topal-interpreter/se-requirements.md).

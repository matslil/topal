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

Topal is currently a design project. It does not yet provide a compiler,
interpreter, or stable language release.

## Repository layout

- [`docs/`](docs/) contains the human-readable language design and is the
  authority for design intent.
- [`decisions.md`](decisions.md) records settled fundamental design decisions.
- [`FUTURE.md`](FUTURE.md) records deliberately deferred work.
- [`se/`](se/) contains system-engineering goals, requirements, validation, and
  traceability information.
- `spec/` will contain the normative formal language specifications.
- [`agents/`](agents/) describes the agent roles used to evolve and verify the
  project.
- `src/` will contain the interpreter, compiler, linter, other tools, and their
  tests once implementation begins.

Repository-wide agent instructions live in [`AGENTS.md`](AGENTS.md). Each implemented
tool will carry its tool-specific requirements in an `se-requirements.md` file
in that tool's directory.

## Build

Topal does not have an implementation to build yet. Build instructions will be
added with the first implementation code.

## Use

There is no build result to use yet. Interpreter, compiler, and tool usage will
be documented when those tools are introduced.

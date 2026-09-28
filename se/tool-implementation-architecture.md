# Tool implementation architecture

This document refines `TOPAL-REQ-SHARED-001` into implementation boundaries
for the source interpreter and native compiler. It explains the dependency
rules future changes must preserve. Language meaning remains owned by `docs/`
and `spec/`; these boundaries organize implementations without adding tool-
specific semantics.

## Architectural drivers

The tools need to share syntax, diagnostics, module identity, and semantic
decisions while retaining different execution strategies. The interpreter is a
deterministic source execution engine. The compiler consumes a checked program
and produces a qualified native artifact. Their host adapters may share source
discovery, but neither tool may call through the other to define semantics.

The implementation therefore follows four principles:

1. **Dependency inversion at host boundaries.** Filesystem, terminal, LLVM, and
   artifact publication code surrounds semantic processing; semantic code does
   not reach outward to drive those adapters.
2. **Interface segregation.** Consumers import the interpreter, compiler,
   module, or tracing facade they need instead of a single flat language API.
3. **Functional core and imperative shell.** Parsed and checked values cross
   explicit stage boundaries. Command-line parsing, file reads, process
   execution, and output publication stay in thin orchestration layers.
4. **Make invalid stage order difficult.** Distinct types represent source
   modules, checked compiler programs, lowered LLVM modules, and published
   artifacts. Raw strings are source or display values, not interchangeable
   pipeline states.

## Shared layer

`topal-language` is the semantic implementation shared by source-level tools.
Its public API is divided into scoped facades:

```text
topal_language::interpreter  Session, Value, reversible execution
topal_language::compiler     checked compiler model and analysis entry points
topal_language::modules      source-module discovery and loading
topal_language::tracing      stable semantic trace events and sinks
```

The crate may retain compatibility re-exports while consumers migrate, but new
tool code uses the narrow facade. Facades expose owned semantic data and
operations, not internal evaluator or analyzer helper types.

Module discovery is a shared adapter because source identity, facade selection,
ordering, and excluded package files must not diverge between tools. A
`SourceModule` is the value passed from discovery to semantic analysis. The
filesystem selector returns deterministic identity order and attaches path
context to I/O failures. The compiler does not independently walk the library
tree.

## Interpreter boundary

The interpreter is organized as an application service around the shared
`Session` execution core:

```text
CLI / build / debugger adapter
            |
            v
Interpreter application service
            |
            v
topal_language::interpreter::Session
```

The service accepts explicit source, source identity, library root, and optional
application input. It returns a semantic `Value` or a structured application
error. It does not print, read standard input, or terminate the process. The
CLI owns argument parsing, terminal interaction, rendering, and exit status.
Interactive incremental input remains a CLI policy over a persistent `Session`.

This is an application-service boundary, not a second evaluator. All language
semantics and traces continue to come from the shared session.

## Compiler boundary

The native compiler is a staged pipeline:

```text
source + CompileOptions
        |
        v
module selection adapter
        |
        v
checked frontend program
        |
        v
LLVM backend -> LlvmModule
        |
        v
LLVM toolchain and artifact publisher -> NativeArtifactMetadata
```

The pipeline facade owns stage order and error translation. Module selection is
delegated to the shared module adapter. The frontend owns semantic acceptance
and produces only the checked compiler model. The backend owns target-specific
lowering and returns a typed `LlvmModule`; it does not invoke tools or publish
files. The toolchain verifies, assembles, links, and atomically publishes the
requested output and metadata.

Backend implementation modules may share private helpers, but checked semantic
types must not depend on LLVM spelling, target layouts, or artifact paths.
Runtime fragments are backend resources selected by checked operations, never
an alternate source of language semantics.

## Error ownership

Each boundary preserves the most useful error type:

- source and semantic rejection uses the shared `Diagnostic` with stable spans;
- module and application adapters attach the path and originating I/O error;
- LLVM discovery and process failures remain tool errors with command context;
- publication errors identify the requested artifact path.

Adapters may render these errors for humans, but lower layers do not format a
complete CLI message or write it to a stream.

## Testing and change rules

Unit tests belong beside the stage whose invariant they exercise. Cross-stage
tests use the public facade. Interpreter/compiler equivalence remains covered by
the shared source corpus, and every memory-intensive validation runs through
the bounded repository runner.

When adding a feature, agents should ask in order:

1. Is this source, syntax, semantic, execution, lowering, platform, or adapter
   policy?
2. Which narrow facade owns it?
3. Can the change be tested before crossing the next stage boundary?
4. Does any duplicated decision now exist in another tool?

Do not introduce a trait merely to rename one concrete function. Add an
abstraction when it enforces a dependency direction, represents a real stage,
or permits a host adapter to vary independently. Do not expose analyzer,
evaluator, LLVM, or filesystem implementation details solely to make a test
convenient.

## Recorded assessment

The September 2026 audit found three structural problems addressed by the
initial modularization:

- the flat `topal-language` export surface mixed unrelated tool capabilities;
- interpreter script/application orchestration lived in the CLI process shell;
- compiler module discovery and all compilation stages were coupled in one
  public function, with LLVM represented by an unqualified `String`.

The large evaluator, checked-model analyzer, and LLVM emitter remain internally
complex by necessity. Future extraction should follow semantic responsibility
(for example values, calls, containers, debug metadata, or runtime selection),
not arbitrary line-count splits. This avoids circular private modules and
procedural abstractions that hide rather than reduce coupling.

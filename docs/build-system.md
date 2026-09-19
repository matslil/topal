# Incremental builds and tests

`topal-build` executes an explicit build and test graph while ordinary Topal
source in `std build graph` decides which units are affected. Tests are graph
units like compilation and generation actions, so a production change selects
every directly or indirectly dependent test.

The initial graph is declared in `topal-build.json`:

```json
{
  "schema": 1,
  "units": [
    {
      "id": "compile-core",
      "kind": "build",
      "inputs": ["src/core.t"],
      "outputs": ["objects/core.geir"],
      "dependencies": [],
      "command": ["compile-core", "src/core.t"]
    },
    {
      "id": "test-core",
      "kind": "test",
      "inputs": ["tests/core.t"],
      "outputs": [],
      "dependencies": ["compile-core"],
      "command": ["test-core", "tests/core.t"]
    }
  ]
}
```

Units appear in dependency order. Commands run directly, without a shell, with
`TOPAL_SOURCE_ROOT` and `TOPAL_BUILD_ROOT` set. Inputs are relative to the
source root; outputs are relative to the build root. Commands that declare
outputs must create all of them before successful state is recorded.

## In-tree builds

The source root defaults to the current directory and the build root defaults
to `.topal-build` beneath it:

```console
topal-build
```

The standard library used to execute the policy defaults to `library` beneath
the source root. An installed toolchain can supply another location with
`--library-root`.

## Out-of-tree builds

Select an independent build root explicitly:

```console
topal-build --source-root project --build-root output/project \
  --library-root /opt/topal/library
```

No declared output or persistent state is written beneath `project` in this
form. Source and build roots do not contribute to graph identities, so moving
the output tree does not change dependency selection.

Use `--dry-run` to print selected units without executing them or advancing
state.

## Change and dependency behavior

The state database records exact input modification timestamps. Any mismatch,
including a modification followed by a content reversion, changes the owning
unit. Missing outputs also select their producer. A manifest timestamp change
conservatively selects the whole graph.

The Topal policy repeatedly follows `(dependency, dependent)` edges. Therefore
a changed library selects its consumers and their tests, while changing only a
test selects that test and not production units. Failed commands and missing
declared outputs leave the previous state unchanged.

The first version operates at explicit manifest-unit granularity. A unit may be
as small as one declaration, but automatic declaration/identifier dependency
extraction is not yet implemented. Filesystem watchers, observed dynamic
dependencies, remote caching, distributed execution, and native sandboxing are
also later increments.

## Native compilation

`topalc` is the ahead-of-time compiler. Its first native target is
`x86_64-unknown-linux-gnu`; selecting another triple is rejected until that
platform has its own qualified lowering and test baseline. The normal command
creates a position-independent ELF executable:

```console
topalc -O0 -g -o hello hello.t
```

`--emit llvm-ir` retains the compiler's LLVM input for inspection and
`--emit object` stops before linking. These implementation formats do not
replace Topal's versioned library interface and generic metadata.

Valid `lang disable-warning` and structured `lang disable-diagnostic` controls
are handled statically. They produce no LLVM instruction or runtime dependency;
malformed control stacks remain shared source diagnostics, and language errors
cannot be disabled by these controls.

Unoptimized native output carries DWARF 5 source, function, parameter, and
local-variable information. Load the bundled GDB value printers before a
debugging session so private runtime values such as arbitrary-precision `Int`,
exact `Rational`, `Range Int`, `Range Rational`, and active nominal
`Union`/`Variant` alternatives are displayed in source form:

```console
gdb -ex 'source src/topal-compiler/gdb/topal.py' ./hello
```

Specialized `Scope` function parameters remain visible as their source
namespace value. Any immutable namespace data threaded through the private
call boundary is also available as a named argument for diagnosis; it is not a
runtime namespace table or a public environment layout.

Within a single-source application, `use root` (or `use` of a retained root
alias) is resolved statically and preserves the namespace snapshot at an
optional binding. It neither flattens members nor performs a runtime filesystem
lookup. External package and library paths remain tied to the future versioned
interface-metadata pipeline rather than ambient host discovery.

A source-root v0.1 function `Interface` and its direct implementation are also
checked statically. `topalc` retains the nominal operation shapes and the exact
implementing declaration identities during checking, then emits ordinary
direct private calls. The interface itself has no runtime object, vtable, or
debug value; GDB shows the implementation function and its values. Source
visibility does not yet publish a compiled-library interface: that remains part
of the separately versioned interface-metadata pipeline.

A directly applied nested lexical function appears as its own source frame.
Represented immutable values captured from its enclosing invocation appear as
named arguments after the declared parameters, so GDB can inspect both without
a foreign closure runtime or an opaque environment object. An exact
nonrecursive, nonoverloaded nested function may also escape its factory through
a compiler-private `Function`, Tuple, Record, exact `Optional Function`, or an
exact selected nominal `Union`/`Variant` Function payload result. It may also
escape as the successful payload of an exact arithmetic `Result Function`;
the Error path retains the original structured Error. An exact finite
`List Function` may carry source-ordered identities and capture snapshots
through the same private boundary. An exact fixed-size `Array Function`
collected from such a List may carry the same identities and snapshots through
private parameters and results. An exact nonempty `Map (String, Function)` may
likewise retain one callable per exact String key across its collision policy,
private parameters, and results. The result carries the
same observation value and immutable capture snapshot, and later application
remains a direct specialized call. An
`Optional Function` stores only the ordinary `Some`/`None` representation and
Function observation value. A Function-bearing nominal Sum retains its normal
tag and declaration-ordered payload slots. A `Result Function` uses the
existing success/Error pointer representation and boxes only the Function
observation on success. In all cases capture snapshots stay in the same hidden
private boundary transport used by other exact Function results, with semantic
Optional-payload, Sum-alternative, Result-success, zero-based List-entry,
zero-based Array-entry, or exact String-keyed Map-value paths. A `List Function`
node stores the ordinary Function observation and next pointer; an Array adds
only its ordinary count and entries pointer. A Map retains its ordinary count,
String key, Function observation, and next-pointer representation. Capture
snapshots are not embedded in any source value. Separate factory invocations
retain separate snapshots. Recursive,
overloaded, opaque,
dynamically selected, persistently stored, and published nested callables remain
outside this private boundary; no public callable ABI is defined.

Ordinary `List Boolean` values use another compiler-selected private node
interpretation: the Boolean payload occupies the first byte and the remaining
List pointer stays naturally aligned at byte eight. Construction, structural
equality, complete List decisions, count, emptiness, private passage, display,
and debugging retain the source classifier. This representation is neither a
type-erased generic container nor a compiled-library ABI; future library
metadata describes the semantic element type, ownership, lifetime, and target
adapter independently of the node offsets.

The earlier `List Effect` node foundation now also supports complete
decomposition, structural equality, count, and emptiness across private
parameters/results and aggregate fields. Because this increment admits only
the sealed canonical empty Effect row, equality compares List length; future
nonempty effect identities require distinct semantic metadata and must not
inherit that private specialization accidentally.

Ordinary `List Comparison` values use a private 16-byte node with the existing
closed i32 comparison carrier and a naturally aligned remaining pointer.
Construction, structural equality, complete decisions, count, emptiness,
private passage, display, and debugging preserve the three source alternatives.
This remains an internal executable representation; future library metadata
records the semantic enum mapping and target adapter rather than publishing
node offsets or compiler helper symbols.

Ordinary `List ErrorCode` values currently retain the closed
`lang arithmetic ArithmeticErrorCode` vocabulary in private i32/pointer nodes.
The four qualified identities remain distinct through equality, decomposition,
display, and debugging. The private tags do not identify future vocabularies;
compiled-library metadata must name the vocabulary and alternatives canonically
and independently of the selected target representation.

Ordinary `List Unit` values retain Unit's zero-information source identity in
private i8/pointer nodes. Since `()` is the only entry value, structural
equality compares List length, while decisions, display, DWARF, and GDB still
preserve the distinction between an entry carrying Unit and `Empty`. This
private carrier is not completion evidence and must never be conflated with
`Completed`, `Effect`, or a public library representation.

Ordinary `List Completed` values use a separate private i8/pointer
specialization. Their singleton equality is also length equality, but each
entry retains explicit completion evidence rather than Unit's absence of a
dependency. Private function results and aggregate fields therefore keep the
existing typed i8 carrier, and future library metadata must preserve Completed
identity even when its current physical node resembles another singleton type.

Ordinary `List Type` values use private i32/pointer nodes whose payload selects
one of the seven closed fundamental Type identities. Equality, decisions,
display, DWARF, and GDB preserve the canonical identity rather than exposing a
host-language descriptor or LLVM type. Future library metadata names the
fundamental identity set and its target adapter without publishing the private
numeric mapping, node offsets, or compiler helpers.

Ordinary Lists of payload-free nominal Enums also use private i32/pointer
nodes, but every List retains its exact declaration identity and ordered source
alternatives. A common private runtime may compare declaration-local tags only
after checking has established the same nominal element type. Future library
metadata records the declaration identity and alternatives, never the numeric
tags, node offsets, or runtime helper as a portable enum or container ABI.

Ordinary Lists of admitted nominal modular values use private pointer/pointer
nodes and keep each canonical arbitrary-precision Int object intact. Equality
compares canonical representatives only after checking proves the same modular
declaration; it never narrows, wraps, or treats the pointer as a machine integer.
Future library metadata retains the declaration, signedness, exact bounds, and
target adapter without publishing Int/List layouts or compiler helpers.

Ordinary `List Optional Int` values keep each existing Optional header pointer
in a private pointer/pointer node. List equality delegates element comparison to
canonical Optional-Int equality; future metadata retains both constructors and
the Int payload contract without publishing either private layout.

Ordinary `List String` values use the same private pointer-payload node shape
already selected for contextual String Lists, now across private boundaries,
structural equality, complete decisions, count, and emptiness. Equality delegates
to canonical preserved-sequence String equality. The node remains an internal
representation; future library metadata retains the semantic element type and
target adapter without publishing compiler helper names or physical offsets.

Ordinary `List Character` values retain their distinct constrained source type
while reusing the private String-descriptor node shape and exact preserved-
sequence comparison. Multi-scalar user-perceived characters are never reduced
to code points or copied into a host text representation. Debug information and
future library metadata must preserve the Character classifier, constraint
evidence, ownership, lifetime, and target adapter independently of this private
layout reuse.

Ordinary `List Nat` values likewise retain a distinct refined source type while
reusing the exact arbitrary-precision Int pointer node and finite equality/count
runtime. Construction validates nonnegativity, and the representation also
retains `+Infinity` exactly; it never introduces machine-unsigned width,
overflow, truncation, or wrapping. Debug information and future library
metadata preserve Nat evidence independently of the private Int-compatible
layout.

Ordinary `List Rational` values use a private descriptor-pointer node and a
conditionally linked finite equality/count fragment. Equality delegates to the
canonical exact Rational comparator, including infinities; no floating-point,
host numeric library, or public layout is introduced.

Topal executables are freestanding with respect to other language runtimes.
They do not acquire a C or C++ standard library, process-startup object, or
dynamic loader dependency merely because the compiler uses LLVM. The Linux
x86-64 platform implementation owns `_start` and implements its initial
standard output and process termination operations directly over the qualified
Linux system-call interface. Future operating-system services belong in
versioned Topal platform libraries with typed effects and failures, not hidden
calls into a foreign standard library.

This does not prohibit an explicitly declared foreign interface in a future
language revision. Such an interface will use checked target adapters and a
concrete published ABI rather than exposing the compiler's private value
representation.

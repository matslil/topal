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

A directly applied nested lexical function appears as its own source frame.
Represented immutable values captured from its enclosing invocation appear as
named arguments after the declared parameters, so GDB can inspect both without
a foreign closure runtime or an opaque environment object. This is a private
compiler boundary; the nested function value cannot yet escape or define a
published callable ABI.

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

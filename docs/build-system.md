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

# Topal examples

These files are runnable programs for the currently implemented language
subset. They are both learning material and automated compatibility fixtures.

Programs under `language/` demonstrate language features independently of the
tool which consumes them. The interpreter, source debugger, language server,
and future compiler conformance suites share these source files rather than
maintaining tool-specific copies. Run one with the interpreter from the
repository root using:

```sh
cargo run -q -p topal-interpreter --bin topal -- examples/language/exact-arithmetic.t
```

Every language-feature increment adds to a related shared example or introduces
a coherent new one. The interpreter executes every file, the language server
opens every file to check diagnostics and editor presentation, and applicable
debugger scenarios select the same file.

Source files remain under `debugger/` only when they demonstrate debugger
control, history, or failure behavior rather than a language feature. Debugger
command scripts are necessarily tool-specific and also remain there.

Programs under `language-diagnostics/` are shared malformed or failing language
examples. Tools use them to compare diagnostics and retained failure behavior;
they are kept outside the successful `language/` corpus intentionally.

Programs under `linter/` are Topal rule modules rather than applications. They
explicitly select the `lint` language variant and are admitted by `topal-lint`
before contained rule execution; the language server opens the same files for
variant-aware editing support.

Programs under `data-transfer/` are executable, self-checking application
policies built over the source standard library. `firewall.t` introduces
bounded layered views. `packet-filter.t` develops that idea into a fail-closed,
batch-oriented filter with an established-flow fast path, ordered policy, IPv4
and IPv6 equivalence, and explicit slow path. `rest-controller.t` demonstrates
a REST API whose typed controller functions are embedded in an `Interface`
implementation while HTTP routing and response policy remain reusable.

These examples simulate semantic inputs so they run on every host. They do not
claim to open sockets, attach XDP programs, or benchmark the current runtime.
Native endpoint adapters can supply the same inputs without moving portable
policy out of Topal.

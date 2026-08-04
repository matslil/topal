# Topal examples

These files are runnable programs for the currently implemented language
subset. They are both learning material and automated compatibility fixtures.

Run an interpreter example from the repository root with:

```sh
cargo run -q -p topal-interpreter --bin topal -- examples/interpreter/exact-arithmetic.t
```

Every language-feature increment adds to a related example or introduces a
coherent new one. The interpreter executes every file, and the language server
opens every file to check diagnostics and editor presentation.

# Topal linter

`topal-lint` is the batch interface to shared Topal diagnostics and the
best-practice catalog. The first increment validates source through the shared
source/syntax layers and provides deterministic catalog selection:

```text
topal-lint source.t
topal-lint --format json source.t
topal-lint --list
topal-lint --explain "lang best-practice task state-machine"
topal-lint --enable "namespace:lang" --disable "tag:lang best-practice tag style" --list
```

An external generated catalog is loaded only when named with `--catalog`.
Stable identity collisions and unsupported schemas are rejected before source
analysis. Installing a library never causes this executable to discover,
enable, or execute its lint code implicitly.

Selectors have three specificity levels: exact identity overrides namespace,
which overrides tag. Later command-line settings win within one level. A
severity of `off` disables the matching entry. Catalog entries without a
reviewed rule attachment never produce a source finding. The bootstrap
`builtin` engine contains the first decidable rule while the `lint` Topal
language variant is being implemented; its attachment is versioned and uses
the same contained dispatch boundary as future Topal rules.

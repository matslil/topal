# Reference-documentation generator requirements

## TOPAL-DOC-INPUT-001 — Explicit source selection

`topal-doc` shall accept one or more explicit source files or directories. It
shall include only direct Topal files for a directory unless `--recurse` is
present, and shall never add an implicit standard-library input.

## TOPAL-DOC-RST-001 — reStructuredText output

The tool shall create deterministic reStructuredText declaration reference
files and an index in the requested output directory. Entries shall preserve
overloads, source syntax, declaration prose, and documented parameter details.

## TOPAL-DOC-LANG-001 — Built-in metadata

`--include-lang` shall add reference material for documented built-in `lang`
identifiers. Without it the tool shall emit no built-in entries.

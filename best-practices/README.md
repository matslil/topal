# Topal best-practice database

Authoritative entries live below `entries/`. Each entry combines a strict
machine-readable `best-practice.json` record with human-authored `guidance.md`
and references shared Topal examples under `examples/`.

The JSON encoding bootstraps the database before the Topal lint language
variant can host its own catalog tooling; it does not replace the semantic
model documented in `docs/best-practices.md` and `spec/best-practices.md`.

Run `cargo run -p topal-best-practices -- generate` after changing an entry.
Run `cargo run -p topal-best-practices -- check` to verify all committed human,
agent, and lint-catalog projections and reject obsolete generated files. A
generated executable attachment embeds its Topal source and SHA-256 digest;
consumers verify the source without reopening the authoritative path. Each
attachment also names the stable revision of the read-only host view supplied
to its entry point.

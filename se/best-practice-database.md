# Best-practice database architecture

This architecture realizes `TOPAL-GOAL-TOOLCHAIN-001`,
`TOPAL-GOAL-EVOLVE-001`, `TOPAL-REQ-SHARED-001`, and
`TOPAL-REQ-BEST-PRACTICE-001`.

## Authoritative and generated layers

```text
authoritative records, guidance, rules, and examples
                         |
                         v
           deterministic validation/generation
              /                  |             \
             v                   v              v
   human reference       agent decision view   lint catalog
```

The repository stores all layers. Generated content carries input digests and
is checked by regeneration. Human or agent presentation cannot become an
independent source of meaning.

## Shared lint pipeline

The linter consumes the shared source, syntax, and semantic layers described in
`se/toolchain-architecture.md`. Rules declare their minimum analysis stage and
receive a versioned read-only view. The linter owns scheduling, containment,
configuration, diagnostics, formatting adapters, and rectification conflict
handling. Rules own only analysis of their declared inputs.

The initial implementation order is catalog validation and querying, generated
human and agent projections, the `lint` language variant, shared diagnostics,
one decidable end-to-end rule, and then assistive architectural rules. Runtime
best-practices later consume an explicitly supplied trace rather than silently
executing an application.

## Risk and trust

Repository-owned rules receive the same limited lint capability as external
rules. Selecting an external database is explicit and does not broaden its
namespace ownership. Rule resource bounds, deterministic ordering, dependency
cycles, malformed output, and failures are checked by the linter. A faulty rule
cannot modify inspected code except through an explicit rectification selected
by the user.

## Evolution

The schema, semantic-view API, best-practice entry, and rule implementation have
separate versions. Compatibility is checked before a rule runs. Stable entry
identity survives presentation changes, deprecation, obsolescence, movement,
and generation format changes.

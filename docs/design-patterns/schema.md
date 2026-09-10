# Pattern record schema

Every catalog entry uses these fields in this order:

| Field | Meaning |
| --- | --- |
| **ID and name** | Stable identifier in this library and the common pattern name. |
| **Good fit** | Situation in which the cited guidance supports using it. |
| **References** | Primary research or official/responsible guidance from which the entry was derived. |
| **Hardware assumptions** | Required or favored hardware; “none” means a semantic pattern portable across targets. |
| **Problem** | Failure, complexity, or performance cost the pattern addresses. |
| **Structure** | The recurring solution shape, at a level independent of one API. |
| **Limitations** | Counter-indications, costs, and properties the pattern does not guarantee. |
| **Core-language support required** | Semantics or static evidence needed to express, verify, or optimize the pattern. Runtime and platform obligations are distinguished explicitly. |

IDs are namespaced by family: `AP` (abstraction/lifecycle), `FD`
(functional/DSL), `MM` (memory/CPU), `CC` (concurrency), `RT` (real-time), `SR`
(safety/robustness), `ST` (security/trust), `DS` (distributed systems), and
`HA` (hardware accelerator/DSP). IDs are never renumbered; removed records keep
a tombstone and point to their replacement.

A reference supports applicability, not universal superiority. Performance
claims remain scoped to the source's workload and target. A language-support
statement distinguishes:

- semantic necessity: the program cannot say the required thing without it;
- optimization evidence: code is expressible but an implementation cannot
  reliably recover enough information for competitive lowering; and
- environment support: a runtime, OS, device, tool, or assurance case must
  provide something outside the core language.

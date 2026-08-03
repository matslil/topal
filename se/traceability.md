# Traceability

This initial matrix connects system goals to core requirements. Specification,
test, and implementation columns will be added as those artifacts are created.

| Goal | Requirements |
| --- | --- |
| `TOPAL-GOAL-COMPOSE-001` | `TOPAL-REQ-MODEL-001`, `TOPAL-REQ-GENERIC-001` |
| `TOPAL-GOAL-SAFE-001` | `TOPAL-REQ-SAFE-001`, `TOPAL-REQ-TOTAL-001`, `TOPAL-REQ-CONC-001`, `TOPAL-REQ-RESOURCE-001` |
| `TOPAL-GOAL-DETERMINISTIC-001` | `TOPAL-REQ-DETERMINISM-001`, `TOPAL-REQ-INTEROP-001` |
| `TOPAL-GOAL-EXPLICIT-001` | `TOPAL-REQ-EFFECT-001`, `TOPAL-REQ-RESOURCE-001`, `TOPAL-REQ-SERIAL-001` |
| `TOPAL-GOAL-ZEROCOST-001` | `TOPAL-REQ-DETERMINISM-001`, `TOPAL-REQ-RESOURCE-001` |
| `TOPAL-GOAL-PRECISE-001` | `TOPAL-REQ-GENERIC-001`, `TOPAL-REQ-SERIAL-001`, `TOPAL-REQ-TOOLS-001`, `TOPAL-REQ-INTEROP-001` |
| `TOPAL-GOAL-EVOLVE-001` | `TOPAL-REQ-TRACE-001`, `TOPAL-REQ-TOOLS-001` |

## Maintenance rules

- Never reuse a retired stable ID for different meaning.
- Record all applicable relationships, not only one convenient parent.
- A requirement without a validating scenario is incomplete.
- A normative specification rule without a requirement or approved design
  source is suspect and must be reviewed.
- A functional test without a specification-rule reference is incomplete.

# Decision-table semantics

## Formal text

### TOPAL-DECISION-BOOLEAN-001 — Complete Boolean decision table

Within the implemented function-body subset, a Boolean subject expression
followed by more deeply indented rules may use `true` and `false` matchers
separated from one-line actions by `then`, plus `otherwise action`. The table shall
be complete by containing `otherwise` or both Boolean literal matchers.

The subject shall be evaluated exactly once. Rules shall be considered in
source order, the first matching rule shall be selected, and only its action
shall be evaluated. Every action shall produce a value compatible with the
function's result path. Test traces and debugger history shall expose each
considered rule and its match result followed by the selected rule.

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

### TOPAL-DECISION-COMPARISON-001 — Comparison decision matchers

A decision rule may begin with `=`, `/=`, `<`, `>`, `<=`, or `>=` followed by
an operand expression and `then action`. The matcher shall compare
the table's once-evaluated subject as its left operand with the matcher's
evaluated operand as its right operand, using the corresponding ordinary
comparison rule. A comparison table in this subset shall contain `otherwise`.

Comparison rules shall be considered in source order. Evaluation shall stop at
the first true comparison, and only that rule's delayed action shall execute;
if none is true, the `otherwise` action shall execute. Test traces and debugger
history shall retain both the comparison operation reason and table selection.

### TOPAL-DECISION-OPERAND-EXPRESSION-001 — Structural matcher separator

For a comparison matcher, the first `then` token on its rule line structurally
terminates the complete operand expression. Tokens before it shall be parsed
using ordinary expression application and grouping, without treating `then` as
a callable operand. The resulting expression shall be evaluated only when that
rule is considered.

Diagnostics for a missing `then` shall remain on the matcher rule and shall not
consume a separator from a later rule. LSP semantic tokens shall classify
`then` as a keyword independently of the operand expression it terminates.
